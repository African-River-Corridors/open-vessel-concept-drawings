"""The evaluation corpus and its independent oracle (issue #184, phase 0, part 1).

The oracle (`evaluation/oracle/box_oracle.py`) never calls the engine. These tests check it against
closed forms, check that the case files hold exactly what it prints, and check that the engine's
`box_condition` agrees with it on the base cases.
"""
import math
import subprocess
import sys
from pathlib import Path

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "evaluation" / "oracle"))
import box_oracle as oracle  # noqa: E402

from vessel_designer.core.gas_arrangement import fit  # noqa: E402
from vessel_designer.core.gas_stability import assess, box_gz  # noqa: E402

sys.path.insert(0, str(ROOT / "evaluation"))
from engine import box_condition  # noqa: E402

CASES = sorted((ROOT / "evaluation" / "cases").glob("*.yaml"))
KEYS = ["area_0_30", "area_0_40", "area_30_40", "gz_30_plus", "angle_max_gz", "gm"]
ENGINE_NAMES = ["Area 0–30°", "Area 0–40°", "Area 30–40°",
                "GZ at ≥ 30°", "Angle of max GZ", "Initial GM (fluid)"]


def test_the_oracle_gm_is_the_closed_form():
    """Label: sourced - closed form for a box section: KB = T/2, BM = B^2/(12 T), GM = KB + BM - KG."""
    # Box B of the corpus: KB 2.5, BM 196/60, KG 5.5.
    assert oracle.gm(14.0, 5.0, 5.5) == pytest.approx(2.5 + 196.0 / 60.0 - 5.5, abs=1e-12)
    assert oracle.gm(20.0, 4.0, 6.0) == pytest.approx(2.0 + 400.0 / 48.0 - 6.0, abs=1e-12)
    assert oracle.gm(14.0, 5.0, 5.5, 0.1) == pytest.approx(0.267 - 0.1 + (2.5 + 196.0 / 60.0 - 5.5 - 0.267), abs=1e-3)


@pytest.mark.parametrize("phi", [1.0, 5.0, 10.0, 20.0])
def test_the_oracle_gz_is_the_wall_sided_formula_before_deck_edge_and_bilge(phi):
    """Label: sourced - wall-sided formula: GZ = sin(phi) (GM + 1/2 BM tan^2(phi)), exact for a box
    until the deck edge goes under (tan phi = 2 (D - T) / B) or the bilge comes out (2 T / B)."""
    b, d, t, kg = 14.0, 8.0, 5.0, 5.5
    assert math.tan(math.radians(phi)) < min(2 * (d - t) / b, 2 * t / b)
    bm = b * b / (12 * t)
    gm = t / 2 + bm - kg
    expected = math.sin(math.radians(phi)) * (gm + 0.5 * bm * math.tan(math.radians(phi)) ** 2)
    assert oracle.gz(b, d, t, kg, phi) == pytest.approx(expected, abs=1e-9)


def test_the_oracle_waterline_holds_the_displaced_area():
    """Label: property - the submerged area at the oracle's waterline is B x T at every heel angle,
    including past deck-edge immersion and bilge emergence."""
    b, d, t = 14.0, 8.0, 5.0
    for phi_deg in (0.5, 10.0, 25.0, 40.0, 55.0):
        phi = math.radians(phi_deg)
        h = oracle.waterline(b, d, t, phi)
        area = oracle._area_centroid_y(oracle._clip_below(oracle._rotated_box(b, d, phi), h))[0]
        assert area == pytest.approx(b * t, rel=1e-12)


def test_the_oracle_is_independent_of_the_engine_on_gz():
    """Label: property - the engine's box GZ and the oracle's agree to 1e-8 m at every whole degree
    (two different ways of finding the waterline)."""
    for b, d, t, kg in [(20.0, 8.0, 4.0, 6.0), (14.0, 8.0, 5.0, 5.5), (12.0, 6.0, 3.0, 4.0)]:
        for deg in range(1, 61):
            assert box_gz(b, d, t, kg, float(deg)) == pytest.approx(oracle.gz(b, d, t, kg, float(deg)), abs=1e-8)


def test_the_oracle_never_imports_the_engine():
    """Label: property - the oracle shares no code with the engine."""
    text = (ROOT / "evaluation" / "oracle" / "box_oracle.py").read_text(encoding="utf-8")
    assert "vessel_designer" not in text and "ovcd" not in text.replace("evaluation", "")


def test_the_case_files_hold_what_the_oracle_prints():
    """Label: property - `write_cases.py --check` finds no difference."""
    r = subprocess.run([sys.executable, str(ROOT / "evaluation" / "oracle" / "write_cases.py"), "--check"],
                       capture_output=True, text=True)
    assert r.returncode == 0, r.stderr


@pytest.mark.parametrize("path", CASES, ids=lambda p: p.stem)
def test_a_case_file_has_the_documented_fields(path):
    """Label: property - every case has inputs, an expected verdict per criterion and provenance."""
    case = yaml.safe_load(path.read_text(encoding="utf-8"))
    assert case["id"] == path.stem and case["kind"] == "box"
    assert case["provenance"].get("inputs_basis")
    assert case["inputs"]["form"] == "particulars" and "vdesign" in case["inputs"]
    assert list(case["expected"]) == KEYS
    for e in case["expected"].values():
        assert e["verdict"] in ("pass", "fail")
        assert {"rule", "what", "unit", "value", "limit", "margin"} <= set(e)
    p = case["provenance"]
    assert p["verdict_basis"] in ("published", "derived")
    assert {"source", "edition", "locator", "licence", "confidence"} <= set(p)


@pytest.mark.parametrize("path", CASES, ids=lambda p: p.stem)
def test_the_engine_agrees_with_the_oracle_on_the_base_cases(path):
    """Label: property - the engine's verdict on each criterion is the expected one; its values
    are within the engine's grid error of the oracle's (areas 0.002 m rad, angle 1 degree)."""
    case = yaml.safe_load(path.read_text(encoding="utf-8"))
    i = case["inputs"]
    c = box_condition(i["beam_m"], i["depth_m"], i["draught_m"], i["kg_m"], free_surface_m=i["free_surface_m"])
    for key, engine_name in zip(KEYS, ENGINE_NAMES):
        crit = next(x for x in c.criteria if x.name == engine_name)
        exp = case["expected"][key]
        assert crit.ok == (exp["verdict"] == "pass"), key
        tol = {"angle_max_gz": 1.0, "gm": 1e-9, "gz_30_plus": 0.002}.get(key, 0.002)
        assert crit.value == pytest.approx(exp["value"], abs=tol), key


def test_box_condition_is_what_assess_does_for_each_gas_condition():
    """Label: property - a pure refactor: `assess`'s conditions equal `box_condition` on the same
    section, KG and free surface, exactly (the GZ curve, areas, GM and criteria)."""
    f = fit(90.0, 14.8, "LPG_propane", 4.4)
    a = assess(f.engine_result, f.payload_t)
    seen = 0
    for c in a.conditions:
        if not c.gz:
            continue
        b = box_condition(f.beam_m, f.depth_m, c.draught_m, c.kg_m, free_surface_m=c.free_surface_m)
        assert (b.gz, b.gm_fluid_m, b.area_0_30, b.area_0_40, b.area_30_40, b.max_gz_deg, b.criteria) == (
            c.gz, c.gm_fluid_m, c.area_0_30, c.area_0_40, c.area_30_40, c.max_gz_deg, c.criteria)
        seen += 1
    assert seen == 3


def test_box_condition_needs_a_draught_below_the_depth():
    """Label: property - a draught at or above the depth is a refused input, not a curve."""
    with pytest.raises(ValueError):
        box_condition(14.0, 5.0, 5.0, 3.0)


def test_the_generic_barge_reproduces_the_published_gm():
    """Label: sourced - Marlantes, Kim and Hurt, J. Mar. Sci. Eng. 2022, 10(1), 41, section 5.1,
    Tables 3 and 4: L 100, B 20, D 10, d 5, KG 7 m give GM 2.167 m. KB 2.5 + BM 400/60 = 6.667, less KG 7, is 2.167."""
    case = yaml.safe_load((ROOT / "evaluation" / "cases" / "generic_barge.yaml").read_text(encoding="utf-8"))
    i = case["inputs"]
    assert (i["length_m"], i["beam_m"], i["depth_m"], i["draught_m"], i["kg_m"]) == (100.0, 20.0, 10.0, 5.0, 7.0)
    assert i["downflooding_angle_deg"] == 40.0
    cited = case["provenance"]["cited_values"]["gm_m"]
    assert cited == 2.167
    # KB 2.5 + BM 6.667 - KG 7, to the printed three decimals: by hand, by the oracle, by the engine.
    assert round(2.5 + 20.0 ** 2 / (12 * 5.0) - 7.0, 3) == cited
    assert round(oracle.gm(20.0, 5.0, 7.0), 3) == cited
    engine = box_condition(20.0, 10.0, 5.0, 7.0)
    assert round(engine.gm_fluid_m, 3) == cited
    assert case["provenance"]["inputs_basis"] == "published" and case["provenance"]["verdict_basis"] == "derived"
