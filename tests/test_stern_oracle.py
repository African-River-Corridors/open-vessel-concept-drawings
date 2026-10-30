"""The stern oracle of the evaluation harness (issue #355): its limits worked out by hand, the generated case
files are what it writes, and every boundary case moves one number across one limit.

Label: property
"""
import sys
from pathlib import Path

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "evaluation" / "oracle"))
import mutants  # noqa: E402
import stern_oracle as oracle  # noqa: E402
import write_cases  # noqa: E402

EVAL = ROOT / "evaluation"


def case(**over) -> dict:
    i = dict(write_cases.STERN_HULL)
    i.update(over)
    return i


def test_the_limits_by_hand():
    """Label: sourced - ES-TRIN 7.02(2) 250 m; 5.06(1) 13 km/h; SOLAS V/22.1.1 the lesser of 2 L and 500 m;
    RheinSchPV 1.07(2) 350 m. A 100 m craft: SOLAS 200 m; a 260 m craft: 500 m, not 520 m."""
    v = oracle.verdicts(case())
    assert v["es-trin/7.02(2)/blind-zone-unladen"]["limit"] == 250.0
    assert v["es-trin/5.06(1)/speed-13kmh"]["limit"] == 13.0
    assert v["solas/V.22.1.1/blind-zone-light"]["limit"] == 200.0
    assert v["rheinschpv/1.07(2)/blind-zone-laden"]["limit"] == 350.0
    assert oracle.verdicts(case(length_overall_m=260.0))["solas/V.22.1.1/blind-zone-laden"]["limit"] == 500.0
    # SOLAS applies from 45 m.
    assert "solas/V.22.1.1/blind-zone-light" in oracle.verdicts(case(length_overall_m=45.0))
    assert "solas/V.22.1.1/blind-zone-light" not in oracle.verdicts(case(length_overall_m=44.9))


def test_the_redundancy_check_by_hand():
    """Label: sourced - ES-TRIN 28.04(1)(a): over 110 m, a bow thruster with one or more propulsors."""
    key = "es-trin/28.04(1)(a)/propulsion-redundancy"
    assert oracle.verdicts(case(length_overall_m=110.0))[key]["verdict"] == "pass"
    assert oracle.verdicts(case(length_overall_m=110.1))[key]["verdict"] == "fail"
    assert oracle.verdicts(case(length_overall_m=110.1, bow_thruster="tunnel"))[key]["verdict"] == "pass"


def test_the_propeller_ratios_by_hand():
    """Label: sourced - 100 m craft: D 3.0 m of 3.7 m allowed; the aft draught 3.9 m less the axis at 1.5 m is 2.4 m,
    0.8 diameters, over 0.76 (0.71 ducted); the blade load 300 over 400 (500 ducted)."""
    v = oracle.verdicts(case())
    assert v["ovd/propulsion/propeller-fits-stern"]["value"] == pytest.approx(3.0 / 3.7)
    assert v["ovd/propulsion/immersion-light"]["value"] == pytest.approx(0.8 / 0.76)
    assert v["ovd/propulsion/blade-load"]["value"] == pytest.approx(0.75)
    d = oracle.verdicts(case(propeller_ducted=True))
    assert d["ovd/propulsion/immersion-light"]["value"] == pytest.approx(0.8 / 0.71)
    assert d["ovd/propulsion/blade-load"]["value"] == pytest.approx(0.6)


def test_the_speed_check_by_hand():
    """Label: sourced - 18 km/h is 9.72 kn, over the Design Speed of 9.7 kn; 17.9 km/h is under it."""
    key = "ovd/powering/installed-covers-design-speed"
    assert oracle.verdicts(case(speed_deep_water_kmh=18.0))[key]["verdict"] == "pass"
    assert oracle.verdicts(case(speed_deep_water_kmh=17.9))[key]["verdict"] == "fail"


def test_the_case_files_are_what_the_oracle_writes():
    """Label: property"""
    for c in write_cases.STERN_BASES:
        path = EVAL / "cases" / f"{c['id']}.yaml"
        want = yaml.safe_load(write_cases.render(write_cases.case_document(c)))
        assert write_cases.first_difference(yaml.safe_load(path.read_text(encoding="utf-8")), want) is None, c["id"]
    assert mutants.main(["mutants.py", "--check"]) == 0


def test_every_boundary_case_moves_one_number_across_one_limit():
    """Label: property"""
    docs = yaml.safe_load((EVAL / "stern_mutants.yaml").read_text(encoding="utf-8"))["mutants"]
    assert docs
    parents = {c["id"]: c for c in write_cases.STERN_BASES}
    for d in docs:
        m = d["mutant"]
        parent = parents[m["parent"]]["inputs"]
        # Moving the speed moves two Rules; a diameter, a draught or a load, one. Only `moved` may differ.
        changed = [k for k in parent if parent[k] != d["inputs"][k]]
        assert changed == [m["moved"]], (d["id"], changed)
        e = d["expected"][m["criterion"]]
        assert e["verdict"] == m["side"], d["id"]
        assert abs(e["margin"]) == pytest.approx(oracle.STEPS[e["unit"]][0], rel=1e-3, abs=1e-9)
