"""The liquid bulk oracle of the evaluation harness (issue #338): its closed forms worked out by hand,
the generated case files are what it writes, and every boundary case moves one input across one limit.

Label: property
"""
import sys
from pathlib import Path

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "evaluation" / "oracle"))
import mutants  # noqa: E402
import tank_oracle as oracle  # noqa: E402
import write_cases  # noqa: E402

EVAL = ROOT / "evaluation"


def case(**over) -> dict:
    i = {"loa_m": 76.5, "beam_m": 11.4, "depth_m": 3.5, "zone_aft_m": 3.0, "zone_fwd_m": 40.0,
         "double_side_m": 1.2, "double_bottom_m": 0.9, "tanks_in_line": 3, "centreline_bulkhead": False,
         "cofferdam_m": 0.8, "pump_room_m": 0.0, "slop_share": 0.0, "deadweight_t": 1500.0,
         "service_area": "river-only", "adn_vessel_type": "C"}
    i.update(over)
    return i


def test_the_layout_by_hand():
    """Label: sourced - hand calculation. Zone 37 m less two cofferdams of 0.8 m is a block of 35.4 m: three
    tanks of 11.8 m, 9.0 m wide (11.4 - 2 x 1.2) and 2.6 m high (3.5 - 0.9)."""
    lay = oracle.layout(case())
    assert lay["block_m"] == pytest.approx(35.4)
    assert lay["tank_length_m"] == pytest.approx(11.8)
    assert lay["tank_breadth_m"] == pytest.approx(9.0)
    assert lay["tank_height_m"] == pytest.approx(2.6)
    assert lay["tank_volume_m3"] == pytest.approx(11.8 * 9.0 * 2.6)
    assert lay["tank_count"] == 3
    # A centreline bulkhead halves the breadth and doubles the tanks; a slop share takes its part of the block.
    c = oracle.layout(case(centreline_bulkhead=True, slop_share=0.05))
    assert c["tank_breadth_m"] == pytest.approx(4.5) and c["tank_count"] == 6
    assert c["slop_m"] == pytest.approx(0.05 * 35.4)
    assert c["tank_length_m"] == pytest.approx(0.95 * 35.4 / 3)
    assert c["slop_share"] == pytest.approx(0.05)


def test_the_adn_tables_as_printed():
    """Label: sourced - ADN 2025 9.3.2.11.1 (a) and (d): up to 600, 0.3 L B H; 600 to 3,750,
    180 + (L B H - 600) x 0.0635; above 3,750, 380; a tank not longer than 10.00 m to 50.00 m, else 0.20 L."""
    assert oracle.adn_capacity_limit(500.0) == pytest.approx(150.0)
    assert oracle.adn_capacity_limit(600.0) == pytest.approx(180.0)
    assert oracle.adn_capacity_limit(3750.0) == pytest.approx(180.0 + 3150.0 * 0.0635)
    assert oracle.adn_capacity_limit(3751.0) == 380.0
    assert oracle.adn_length_limit(50.0) == 10.0
    assert oracle.adn_length_limit(50.1) == pytest.approx(10.02)
    assert oracle.adn_length_limit(100.0) == pytest.approx(20.0)


def test_the_marpol_limits_by_hand():
    """Label: sourced - MARPOL Annex I Reg. 19.6.1 (h = B/15, at least 0.76 m), 19.6.2 (w = 0.4 + 2.4 DW /
    20,000, at least 0.76 m) and 29.2.3 (3 per cent)."""
    coastal = dict(service_area="coastal-and-river")
    v = oracle.verdicts(case(**coastal, beam_m=14.4, deadweight_t=4000.0, double_bottom_m=0.96, double_side_m=0.88))
    assert v["marpol/19.6.1/double-bottom-depth"]["limit"] == pytest.approx(0.96)
    assert v["marpol/19.6.1/double-bottom-depth"]["verdict"] == "pass"
    assert v["marpol/19.6.2/double-side-width"]["limit"] == pytest.approx(0.88)
    v = oracle.verdicts(case(**coastal, beam_m=9.0, deadweight_t=1000.0, double_bottom_m=0.75, double_side_m=0.75))
    assert v["marpol/19.6.1/double-bottom-depth"]["limit"] == pytest.approx(0.76)
    assert v["marpol/19.6.1/double-bottom-depth"]["verdict"] == "fail"
    assert v["marpol/19.6.2/double-side-width"]["limit"] == pytest.approx(0.76)
    assert v["marpol/29/slop-tank-capacity"]["verdict"] == "fail"          # no slop tank
    # From 5,000 t only the slop rule is evaluated; on a river MARPOL is not selected at all.
    big = oracle.verdicts(case(**coastal, deadweight_t=6000.0))
    assert "marpol/19.6.1/double-bottom-depth" not in big and "marpol/29/slop-tank-capacity" in big
    assert not [k for k in oracle.verdicts(case()) if k.startswith("marpol/")]


def test_a_type_n_vessel_has_the_type_n_rules():
    """Label: property"""
    v = oracle.verdicts(case(adn_vessel_type="N_closed", double_side_m=0.6, double_bottom_m=0.5))
    assert v["adn/9.3.3.11.7/wing-width-min"]["verdict"] == "pass"
    assert v["adn/9.3.3.11.7/double-bottom-depth"]["verdict"] == "pass"
    assert not [k for k in v if k.startswith("adn/9.3.2")]
    c = oracle.verdicts(case())
    assert c["adn/9.3.2.11.7/wing-width-min"]["limit"] == 1.0
    assert "adn/9.3.2.11.7/double-bottom-min" in c and "adn/9.3.3.11.7/wing-width-min" not in c


def test_the_oracle_shares_no_code_with_the_engine():
    """Label: property"""
    text = (ROOT / "evaluation" / "oracle" / "tank_oracle.py").read_text(encoding="utf-8")
    assert "ovd" not in text
    imports = [ln for ln in text.splitlines() if ln.startswith(("import ", "from "))]
    assert all(ln.split()[1] in ("sys", "pathlib", "__future__", "yaml") for ln in imports), imports


def test_the_base_case_files_and_the_boundary_cases_are_what_the_oracle_writes():
    """Label: property"""
    for c in write_cases.TANK_BASES:
        path = EVAL / "cases" / f"{c['id']}.yaml"
        assert write_cases.first_difference(yaml.safe_load(path.read_text(encoding="utf-8")),
                                            yaml.safe_load(write_cases.render(write_cases.case_document(c)))) is None
    got = yaml.safe_load((EVAL / "tank_mutants.yaml").read_text(encoding="utf-8"))
    want = yaml.safe_load(mutants.tank_text())
    assert write_cases.first_difference(got, want) is None


def test_every_boundary_case_moves_one_input_across_one_limit_by_its_step():
    """Label: property"""
    docs = yaml.safe_load((EVAL / "tank_mutants.yaml").read_text(encoding="utf-8"))["mutants"]
    parents = {p.stem: yaml.safe_load(p.read_text(encoding="utf-8")) for p in (EVAL / "cases").glob("tank_*.yaml")}
    assert len(docs) > 150
    seen = set()
    for d in docs:
        m = d["mutant"]
        parent = parents[m["parent"]]
        changed = [k for k in parent["inputs"] if parent["inputs"][k] != d["inputs"][k]]
        assert changed == [m["moved"]], (d["id"], changed)
        big, small = oracle.STEPS[d["expected"][m["criterion"]]["unit"]]
        step = big if m["magnitude"] == "big" else small
        margin = d["expected"][m["criterion"]]["margin"]
        assert margin == pytest.approx(step if m["side"] == "pass" else -step, rel=1e-6, abs=1e-9), (d["id"], margin)
        assert d["expected"][m["criterion"]]["verdict"] == m["side"], d["id"]
        seen.add((m["criterion"], m["side"], m["magnitude"]))
    # Every evaluated Rule of a base case has a boundary case on each side, with each step.
    for rule in {r for p in parents.values() for r in p["expected"]}:
        for side in ("pass", "fail"):
            for mag in ("big", "small"):
                assert (rule, side, mag) in seen, (rule, side, mag)


def test_the_expected_verdicts_of_the_bases_cover_known_good_and_known_bad():
    """Label: property"""
    verdicts = {}
    for p in (EVAL / "cases").glob("tank_*.yaml"):
        for rule, e in yaml.safe_load(p.read_text(encoding="utf-8"))["expected"].items():
            verdicts.setdefault(rule, set()).add(e["verdict"])
    assert any(len(v) == 2 for v in verdicts.values())
    assert {"fail"} <= verdicts["marpol/29/slop-tank-capacity"]
