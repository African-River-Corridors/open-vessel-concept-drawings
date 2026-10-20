"""The Type C tank calibration cases (issue #292): run through the tank maths only, held out, banded.

`calibration/tank_cases.yaml` and `tank_bands.yaml` follow the conventions of `cases.yaml` and
`bands.yaml`: every case is held out, cites a listed web source, keeps the source text as published,
and a band miss is a warning, never a failure.
"""

import re

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
        assert c["source"].startswith(f"{sid} — ") and re.search(r"accessed 2026-\d\d-\d\d$", c["source"]), c["id"]
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
    the tenth of a millimetre. The run uses the engine's S690QL1 stress, the IGC 4.23.3.1 limit Rm/3 =
    256.67 MPa since issue #313: 14.52 mm by hand (first pass 1.58 x 2365 / 255.719 = 14.613 mm, second
    1.58 x 2350.387 / 255.719 = 14.522 mm), 15.5 mm with the allowance, rounded up to 0.5 mm.

    Label: sourced - Senjanovic et al., DESIGN 2006, river LPG tank: 4730 mm outer diameter, 15.8 bar, S690QL1 285 MPa, cylinder 13.1 mm; IGC 4.23.3.1 (Rm 770 MPa over 3); 13.08 mm and 14.52 mm are hand calculations
    """
    from vessel_designer.core import pressure_vessel as pv
    r, p = 4730.0 / 2.0, 1.58
    at_paper = pv.cylinder_wall_thickness(p, r - pv.cylinder_wall_thickness(p, r, 285.0, 1.0), 285.0, 1.0)
    assert at_paper == pytest.approx(13.08, abs=0.005)
    assert abs(at_paper - 13.1) <= 0.05
    row = _run_tank(next(c for c in _cases() if c["id"] == "senjanovic-2006-river-lpg"))
    assert row.required_mm == pytest.approx(14.522, abs=0.0005)
    # The engine's wall adds the allowance and rounds up to 0.5 mm.
    assert row.wall_mm == 15.5


def test_an_accepted_miss_is_on_a_figure_outside_its_band_and_the_warning_gives_the_reason():
    """Label: property
    """
    checks = {(b.case, b.figure): b for b in _assess_tanks()}
    accepted = [(c["id"], fig, why) for c in _cases() for fig, why in c.get("accepted_miss", {}).items()]
    assert accepted, "the river LPG wall miss of issue #313 is recorded"
    for case, fig, why in accepted:
        assert why.strip(), (case, fig)
        assert not checks[(case, fig)].inside, f"{case} {fig}: inside its band, so nothing to accept"
    lines = _tank_warnings_for(list(checks.values()))
    for case, fig, why in accepted:
        assert any(case in x and f"accepted: {why}" in x for x in lines), (case, fig)
    assert "accepted: " in report()


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


def test_every_tank_weight_record_cites_listed_sources_and_is_not_run_without_a_datum():
    """Issue #317: a weight record names every source it uses, keeps the published text, says what the
    weight includes, and is never run without a stated diameter and length basis.

    Label: property
    """
    web = {w["id"]: w for w in yaml.safe_load((CAL / "sources.yaml").read_text(encoding="utf-8"))["web_sources"]}
    records = _load_tank_cases()["tank_weight_records"]
    assert len(records) >= 2
    for c in records:
        ids = re.findall(r"\b(P\d+) — ", c["source"])
        assert ids and all(i in web for i in ids), c["id"]
        assert c["published"].strip() and c["weight_includes"].strip() and str(c["held_out"]).strip(), c["id"]
        assert c["confidence"] in CONFIDENCES, c["id"]
        runnable = "diameter_basis" in c and "length_basis" in c
        assert runnable or c["not_yet_runnable"].strip(), c["id"]
        assert not runnable, "a runnable weight record needs the runner first"
    ids_run = {r.case["id"] for r in map(_run_tank, _cases())}
    assert not ids_run & {c["id"] for c in records}


def test_the_weight_per_m3_is_the_published_weight_over_the_published_volume():
    """155 t / 1,350 m3 = 0.1148 and 240 t / 2,500 m3 = 0.0960 t per m3, by hand.

    Label: sourced - tank maker's news release 8 Sep 2020 and Breakbulk News 23 Nov 2022 (1,350 m3, 155 t); FE software maker's customer story (2,500 m3, 240 t); the ratios are hand calculations
    """
    by_id = {c["id"]: c for c in _load_tank_cases()["tank_weight_records"]}
    assert calibration._tank_weight_per_m3(by_id["lng-bunker-barge-tank-1350"]) == pytest.approx(0.11481, abs=5e-6)
    assert calibration._tank_weight_per_m3(by_id["lng-bunker-vessel-bilobe-2500"]) == pytest.approx(0.0960, abs=5e-6)
    lo, hi = calibration._tank_weight_range()
    assert (round(lo, 3), round(hi, 3)) == (0.096, 0.115)
    assert "Tank weight per m³" in report()
