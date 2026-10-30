"""The concept design report script builds every part from the run (issue #727).

`tools/concept_report.py` turns one `ovd run --json` Option into an Open Report Designer document. Risk
stopped: a report part says something the run does not, a typed figure creeps in, a warning Rule drops out
of the compliance matrix, or the resistance method is named as something the engine does not run.

The fixture is the gas example's stored run, `examples/gas-140x29-coastal-lng/results.json`.

Label: property. Each test checks the report against the run it is built from; no number is pinned.
"""
import importlib.util
import json
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location("concept_report", ROOT / "tools" / "concept_report.py")
cr = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(cr)

RUN = json.loads((ROOT / "examples" / "gas-140x29-coastal-lng" / "results.json").read_text(encoding="utf-8"))
A = next(o for o in RUN["options"] if o["name"] == "A basis")
C = next(o for o in RUN["options"] if o["name"] == "C deep")


def test_lightship_items_add_to_the_lightship():
    """Label: property"""
    items = cr.lightship_items(A)
    total = A["results"]["results"]["lightship_t"]
    assert abs(sum(v for _, v, _ in items) - total) < 1e-6
    assert all(v >= 0 for _, v, _ in items)
    table = cr.lightship(A)
    assert table["rows"][-1][1] == cr.n(total)


def test_particulars_come_from_the_run_and_show_the_reference_beside_them():
    """Label: property"""
    one = cr.particulars(A)
    assert [c["title"] for c in one["columns"]] == ["Item", "Value", "Basis"]
    two = cr.particulars(C, ref=A)
    assert [c["title"] for c in two["columns"]] == ["Item", "C deep", "Reference: A basis", "Basis"]
    depth = next(r for r in two["rows"] if r[0] == "Depth, moulded")
    assert depth[1] == f"{C['results']['dims']['depth_m']:,.2f} m"
    assert depth[2] == f"{A['results']['dims']['depth_m']:,.2f} m"


def test_the_steel_table_gives_both_figures_and_their_gap():
    """Label: property"""
    st = A["results"]["cost_masses"]["steel"]
    t = cr.steel(A, "under review")
    assert t["rows"][0][1] == cr.n(st["top_down_t"])
    assert t["rows"][1][1] == cr.n(st["bottom_up_t"])
    assert t["rows"][2][1].startswith(cr.n(st["top_down_t"] - st["bottom_up_t"]))
    assert "under review" in t["caption"]


def test_the_matrix_has_one_row_per_governing_or_warning_rule_with_its_worst_outcome():
    """Label: property"""
    rows = A["results"]["rule_trace"]["rows"]
    want = {r["rule"] for r in rows if r["outcome"] in ("warn", "fail") or (r["outcome"] == "pass" and r.get("governing"))}
    got = cr.governing(A)
    assert sorted(r["rule"] for r in got) == sorted(want)
    for g in got:
        outcomes = [r["outcome"] for r in rows if r["rule"] == g["rule"] and r["outcome"] in ("pass", "warn", "fail")]
        assert g["outcome"] == min(outcomes, key=cr.OUTCOME_ORDER.index)
    m = cr.compliance(A)
    assert [c["title"] for c in m["columns"]] == ["Rule", "Clause", "Requirement", "Value", "Limit", "Outcome", "Verified"]
    assert len(m["rows"]) == len(want)
    assert {r[6] for r in m["rows"]} <= {"yes", "no"}


def test_every_governing_rule_gets_one_paragraph_and_a_note_is_added_where_given():
    """Label: property"""
    gov = cr.governing(A)
    first = gov[0]["rule"]
    text = cr.rule_paragraphs(A, {first: "A note for this rule."})
    paras = text.split("\n\n")
    assert len(paras) == len(gov)
    assert paras[0].endswith("A note for this rule.")
    for p, g in zip(paras, gov):
        name, clause = cr.set_and_clause(g["rule"])
        assert p.startswith(f"**{name} {clause}.**")


def test_the_powering_method_is_named_as_the_engine_runs_it():
    """Label: property"""
    m = cr.POWERING_METHOD
    assert "ITTC-1957" in m and "Cr/Cf" in m
    assert "Holtrop–Raven" not in m and "Holtrop-Raven" not in m
    assert "no hull-form term" in m or "only through the wetted surface" in m


def test_not_evaluated_lists_each_rule_once():
    """Label: property"""
    ne = cr.not_evaluated(A)
    ids = [x.split(":")[0] for x in ne]
    assert len(ids) == len(set(ids))
    assert set(ids) == {r["rule"] for r in A["results"]["rule_trace"]["rows"] if r["outcome"] == "not evaluated"}


def test_the_document_has_the_standard_sections_and_no_project_text():
    """Label: property"""
    doc = cr.build(RUN, "A basis", number="TN-0001", date="2026-10-30")
    assert doc["ord"] == "0.2"
    assert [s["id"] for s in doc["body"]] == ["design", "lightship", "stability", "powering", "rules"]
    assert [a["id"] for a in doc["appendices"]] == ["provenance"]
    assert doc["notice"] == cr.NOTICE
    prov = doc["appendices"][0]["body"][0]
    assert prov["rows"][0][1] == A["results_hash"][:12]


def test_the_cli_writes_a_document_that_ord_accepts(tmp_path):
    """Label: property"""
    run = tmp_path / "run.json"
    run.write_text(json.dumps(RUN), encoding="utf-8")
    out = tmp_path / "doc.json"
    subprocess.run([sys.executable, str(ROOT / "tools" / "concept_report.py"), str(run), "--option", "C deep", "--ref", "A basis",
                    "-o", str(out)], check=True)
    doc = json.loads(out.read_text(encoding="utf-8"))
    assert doc["body"][0]["body"][1]["columns"][2]["title"] == "Reference: A basis"
    ord_bin = shutil.which("ord")
    if ord_bin is None:
        pytest.skip("ord is not on the path")
    subprocess.run([ord_bin, "check", str(out)], check=True)
