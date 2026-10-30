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

from ovd.core.gas_arrangement import fit  # noqa: E402
from ovd.core.gas_stability import assess, box_gz  # noqa: E402

sys.path.insert(0, str(ROOT / "evaluation"))
from engine import box_condition  # noqa: E402

# The box cases; the liquid bulk (`kind: tank`) cases have their own tests in `tests/test_tank_oracle.py`.
CASES = sorted(p for p in (ROOT / "evaluation" / "cases").glob("*.yaml")
               if yaml.safe_load(p.read_text(encoding="utf-8")).get("kind") == "box")
SHORT = ["area_0_30", "area_0_40", "area_30_40", "gz_30_plus", "angle_max_gz", "gm"]
#: The case files key `expected` by rule id (issue #278).
KEYS = ["is2008/A.2.2.1/area-0-30", "is2008/A.2.2.1/area-0-40", "is2008/A.2.2.1/area-30-40",
        "is2008/A.2.2.2/gz-30", "is2008/A.2.2.3/angle-max-gz", "is2008/A.2.2.4/gm0"]
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
    assert "ovd" not in text and "ovd" not in text.replace("evaluation", "")


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
    for short, key, engine_name in zip(SHORT, KEYS, ENGINE_NAMES):
        crit = next(x for x in c.criteria if x.name == engine_name)
        exp = case["expected"][key]
        assert crit.ok == (exp["verdict"] == "pass"), key
        tol = {"angle_max_gz": 1.0, "gm": 1e-9, "gz_30_plus": 0.002}.get(short, 0.002)
        assert crit.value == pytest.approx(exp["value"], abs=tol), key


def test_assess_is_on_the_hull_sections_and_box_condition_stays_the_box():
    """Label: property - since issue #448 `assess` works each condition on the hull's Sections
    Table (the raked bow and the Stern Form) at free trim, so its GZ curve is not the box's at the
    same draught, KG and free surface. `box_condition` stays the prismatic box of the evaluation
    harness: on the same 0 to 60 degree grid, with the same criteria, upright at GZ 0."""
    f = fit(90.0, 14.8, "LPG_propane", 4.4)
    a = assess(f.engine_result, f.payload_t)
    seen = 0
    for c in a.conditions:
        if not c.gz:
            continue
        b = box_condition(f.beam_m, f.depth_m, c.draught_m, c.kg_m, free_surface_m=c.free_surface_m)
        assert [deg for deg, _ in b.gz] == [deg for deg, _ in c.gz]
        assert [k.name for k in b.criteria] == [k.name for k in c.criteria]
        assert b.gz[0][1] == 0.0
        assert b.gz != c.gz
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


FT = 0.3048


def _lcu(name):
    return yaml.safe_load((ROOT / "evaluation" / "cases" / f"{name}.yaml").read_text(encoding="utf-8"))


def test_the_lcu_reproduces_the_printed_km_and_gm():
    """Label: sourced - US Naval Academy, EN400 Principles of Ship Performance, Course Notes, Summer
    2020, chapter 4, problem 33, printed page 4-51 (PDF page 190): KM 27.5, 22.9 and 20.0 ft at drafts
    4, 5 and 6 ft for a 135 x 35 ft box; empty KG 1.9 ft, so GM 25.6 ft."""
    printed = {4.0: 27.5, 5.0: 22.9, 6.0: 20.0}
    b = 35.0
    empty = _lcu("lcu_empty")["inputs"]
    assert empty["beam_m"] == pytest.approx(b * FT, rel=1e-12)
    for draft_ft, km_ft in printed.items():
        by_hand = draft_ft / 2 + b * b / (12 * draft_ft)
        assert round(by_hand, 1) == km_ft
        # The oracle's GM + KG is KM; so is the engine's.
        kg = 1.9 * FT
        assert round((oracle.gm(b * FT, draft_ft * FT, kg) + kg) / FT, 1) == km_ft
        engine = box_condition(b * FT, 7.0 * FT, draft_ft * FT, kg)
        assert round((engine.gm_solid_m + kg) / FT, 1) == km_ft
    assert round(oracle.gm(empty["beam_m"], empty["draught_m"], empty["kg_m"]) / FT, 1) == 25.6
    engine = box_condition(empty["beam_m"], empty["depth_m"], empty["draught_m"], empty["kg_m"])
    assert round(engine.gm_fluid_m / FT, 1) == 25.6


def test_the_lcu_loaded_inputs_follow_from_the_printed_numbers():
    """Label: sourced - same source. Salt water as the notes use it (TPI = Awp/420, 35 ft3 per long ton):
    TPI = 135 x 35 / 420 = 11.25 LT/in (as printed); empty displacement 135 x 35 x 4 / 35 = 540 LT;
    3 x 60 = 180 LT is 16 in, so draft 5.3333 ft; KG = (540 x 1.9 + 180 x 11) / 720 = 4.175 ft."""
    assert 135.0 * 35.0 / 420.0 == 11.25
    delta0 = 135.0 * 35.0 * 4.0 / 35.0
    assert delta0 == 540.0
    draft_ft = 4.0 + 180.0 / 11.25 / 12.0
    assert (delta0 + 180.0) * 35.0 / (135.0 * 35.0) == pytest.approx(draft_ft, abs=1e-12)   # same draft from the volume
    kg_ft = (delta0 * 1.9 + 180.0 * (7.0 + 4.0)) / (delta0 + 180.0)
    assert kg_ft == pytest.approx(4.175, abs=1e-12)
    loaded = _lcu("lcu_loaded")
    i = loaded["inputs"]
    assert i["draught_m"] == pytest.approx(draft_ft * FT, rel=1e-10)
    assert i["kg_m"] == pytest.approx(kg_ft * FT, rel=1e-10)
    assert i["depth_m"] == pytest.approx(7.0 * FT, rel=1e-12)
    assert "downflooding_angle_deg" not in i          # not given by the source
    # The printed KM at 5 ft (22.9) and 6 ft (20.0) bracket the box KM at the loaded draft.
    km = draft_ft / 2 + 35.0 ** 2 / (12 * draft_ft)
    assert 20.0 < km < 22.9
    assert oracle.gm(i["beam_m"], i["draught_m"], i["kg_m"]) / FT == pytest.approx(km - kg_ft, abs=1e-9)
    # The verdict is the oracle's, whatever it is: this hull fails the angle of maximum GZ.
    assert loaded["expected"]["is2008/A.2.2.3/angle-max-gz"]["verdict"] == "fail"
