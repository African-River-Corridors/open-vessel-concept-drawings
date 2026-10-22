"""The evaluation runner, the generated boundary mutants and the CI gate (issue #184, phase 0, part 2;
keyed by rule id, with triage, since issue #278).

A false pass (the engine passes a blocking Rule a case expects to fail) must turn the runner red; a
false fail and a coverage gap are warnings. Each disagreement goes in one triage bucket. These tests
run the real runner as CI does.
"""
import shutil
import subprocess
import sys
from pathlib import Path

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[1]
EVAL = ROOT / "evaluation"
sys.path.insert(0, str(EVAL))
sys.path.insert(0, str(EVAL / "oracle"))
import run as runner  # noqa: E402
import mutants as gen  # noqa: E402
import write_cases  # noqa: E402

#: The oracle's criterion names, and the rule id each one is keyed by in the case files.
SHORT = list(write_cases.RULE_IDS)
CRITERIA = list(write_cases.RULE_IDS.values())
SHORT_OF = {v: k for k, v in write_cases.RULE_IDS.items()}
AREA_0_30, AREA_0_40, GM = CRITERIA[0], CRITERIA[1], CRITERIA[5]
WEATHER = "is2008/A.2.3/weather"


def run_runner(*args):
    return subprocess.run([sys.executable, str(EVAL / "run.py"), *args], capture_output=True, text=True)


def corpus_copy(tmp_path):
    cases = tmp_path / "cases"
    shutil.copytree(EVAL / "cases", cases)
    shutil.copy(EVAL / "mutants.yaml", tmp_path / "mutants.yaml")
    return cases, tmp_path / "mutants.yaml", tmp_path / "results.md"


def test_the_real_corpus_has_no_false_pass_and_its_results_are_current():
    """Label: property - the CI command exits 0: no false pass, and results.md is what the run writes."""
    r = run_runner("--check")
    assert r.returncode == 0, r.stdout + r.stderr
    assert "0 false passes" in r.stdout


def test_a_deliberately_wrong_expected_verdict_turns_the_runner_red(tmp_path):
    """Label: property - a case that expects a criterion to FAIL where the engine passes it is a false
    pass: the runner exits 1 and prints an error annotation."""
    cases, mutants, results = corpus_copy(tmp_path)
    path = cases / "box_a.yaml"
    doc = yaml.safe_load(path.read_text(encoding="utf-8"))
    assert doc["expected"][AREA_0_30]["verdict"] == "pass"
    doc["expected"][AREA_0_30]["verdict"] = "fail"            # deliberately wrong
    path.write_text(yaml.safe_dump(doc, sort_keys=False), encoding="utf-8")
    r = run_runner("--cases", str(cases), "--mutants", str(mutants), "--results", str(results))
    assert r.returncode == 1, r.stdout
    assert "::error title=Evaluation false pass::box_a" in r.stdout
    assert "1 false passes" in r.stdout


def test_a_false_fail_is_only_a_warning(tmp_path):
    """Label: property - a case that expects a criterion to PASS where the engine fails it is a false
    fail: a warning annotation, exit 0 (as a Calibration Band miss is)."""
    cases, mutants, results = corpus_copy(tmp_path)
    path = cases / "box_b.yaml"
    doc = yaml.safe_load(path.read_text(encoding="utf-8"))
    # Box B passes everything; make the case ask for a state the engine fails: move KG across GM.
    doc["inputs"]["kg_m"] = 5.9                                # GM < 0.15: the engine fails the GM
    path.write_text(yaml.safe_dump(doc, sort_keys=False), encoding="utf-8")
    r = run_runner("--cases", str(cases), "--mutants", str(mutants), "--results", str(results))
    assert r.returncode == 0, r.stdout
    assert "::warning title=Evaluation false fail::box_b" in r.stdout
    assert "::error" not in r.stdout


def test_a_coverage_gap_is_only_a_warning(tmp_path):
    """Label: property - with no mutants only the base cases run. Every criterion is expected to pass in
    them except the angle of maximum GZ, which the loaded LCU fails: the other five are coverage gaps
    (warnings, exit 0) and the angle is not."""
    cases, _, results = corpus_copy(tmp_path)
    r = run_runner("--cases", str(cases), "--mutants", str(tmp_path / "none.yaml"), "--results", str(results))
    assert r.returncode == 0, r.stdout
    assert r.stdout.count("::warning title=Evaluation coverage gap") == 5
    gaps = [line for line in r.stdout.splitlines() if "coverage gap" in line]
    assert not any("angle-max-gz" in line for line in gaps)


def test_a_stale_results_file_fails_the_check(tmp_path):
    """Label: property - --check fails when results.md is not what the run writes."""
    cases, mutants, results = corpus_copy(tmp_path)
    shutil.copy(EVAL / "results.md", results)
    results.write_text(results.read_text(encoding="utf-8") + "\nstale\n", encoding="utf-8")
    r = run_runner("--check", "--cases", str(cases), "--mutants", str(mutants), "--results", str(results))
    assert r.returncode == 1 and "Evaluation results stale" in r.stdout


def test_every_criterion_is_covered_from_both_sides_by_a_mutant_outside_the_grid_error():
    """Label: property - each of the six criteria has at least one generated mutant on each side of its
    limit, outside the engine's grid error, for the targeted criterion."""
    docs = yaml.safe_load((EVAL / "mutants.yaml").read_text(encoding="utf-8"))["mutants"]
    for criterion in CRITERIA:
        for side in ("pass", "fail"):
            hits = [d for d in docs if d["mutant"]["criterion"] == criterion and d["mutant"]["side"] == side
                    and not d["mutant"]["grid"] and not d["expected"][criterion]["grid"]]
            assert hits, (criterion, side)
            for d in hits:
                assert d["expected"][criterion]["verdict"] == side


def test_a_mutant_moves_one_input_across_the_limit_by_its_step():
    """Label: property - every mutant differs from its parent in the one input it moves, and the
    targeted criterion's margin is the step (to 1e-6 of it), on the side asked for."""
    docs = yaml.safe_load((EVAL / "mutants.yaml").read_text(encoding="utf-8"))["mutants"]
    parents = {p.stem: yaml.safe_load(p.read_text(encoding="utf-8")) for p in (EVAL / "cases").glob("*.yaml")}
    for d in docs:
        m = d["mutant"]
        parent = parents[m["parent"]]
        changed = [k for k in parent["inputs"] if parent["inputs"][k] != d["inputs"][k]]
        assert changed == [m["moved"]], (d["id"], changed)
        short = SHORT_OF[m["criterion"]]
        big, small = gen.STEPS[short]
        step = big if m["magnitude"] == "big" else small
        margin = d["expected"][m["criterion"]]["margin"]
        assert abs(margin - (step if m["side"] == "pass" else -step)) <= 1e-3 * step + 1e-9, (d["id"], margin)
        assert m["grid"] == (m["magnitude"] == "small" and step < gen.GRID_TOL[short] * 4)


def test_the_grid_error_tolerances_are_above_what_the_engine_shows():
    """Label: property - on every case and mutant the engine's value is within the grid tolerance of
    the oracle's, so a verdict outside the tolerance is one the engine must get right."""
    worst = {k: 0.0 for k in CRITERIA}
    for case in box_cases():
        for k, r in runner.run_case(case)["rows"].items():
            worst[k] = max(worst[k], abs(r["engine_value"] - r["expected_value"]))
    for k, w in worst.items():
        assert w <= gen.GRID_TOL[SHORT_OF[k]], (k, w)
    for k in SHORT:
        assert gen.STEPS[k][0] >= gen.GRID_TOL[k], k


@pytest.mark.slow
def test_the_mutants_file_is_what_the_generator_writes():
    """Label: property - evaluation/mutants.yaml is generated, never edited by hand."""
    r = subprocess.run([sys.executable, str(EVAL / "oracle" / "mutants.py"), "--check"],
                       capture_output=True, text=True)
    assert r.returncode == 0, r.stderr


# --- triage (issue #278): one fixture per bucket ----------------------------------------------------

def box_cases() -> list:
    """The stability (box) cases and mutants: the liquid bulk ones are tested in test_tank_oracle.py."""
    return [c for c in runner.load_cases() if runner.kind_of(c) == "box"]


def _case(name="box_b"):
    return yaml.safe_load((EVAL / "cases" / f"{name}.yaml").read_text(encoding="utf-8"))


def _disagreements(case):
    return runner.evaluate([case])["disagreements"]


def test_triage_a_derived_case_that_disagrees_with_the_engine_is_a_code_bug():
    """Label: property - box B's expected verdicts are the oracle's: a wrong one is the oracle and the
    engine disagreeing, so `code-bug`, and a false pass."""
    case = _case()
    case["expected"][AREA_0_30]["verdict"] = "fail"
    r = runner.evaluate([case])
    assert r["disagreements"] == [{"case": "box_b", "rule": AREA_0_30, "expected": "fail",
                                   "engine": "pass", "bucket": "code-bug"}]
    assert r["per"][AREA_0_30]["false_passes"] == ["box_b"]


def test_triage_a_rule_that_is_not_evaluated_is_a_model_gap_and_never_a_false_pass():
    """Label: property - the weather criterion is `not evaluated`: a case that expects it to fail is a
    `model-gap`, and the run has no false pass."""
    case = _case()
    case["expected"][WEATHER] = dict(case["expected"][GM], verdict="fail")
    r = runner.evaluate([case])
    assert [d["bucket"] for d in r["disagreements"]] == ["model-gap"]
    assert runner.totals(r)["false_passes"] == 0


def test_triage_a_case_that_needs_an_input_the_engine_lacks_is_a_model_gap():
    """Label: property - a box case with an input the box trace does not read is a `model-gap`."""
    case = _case()
    case["inputs"]["wind_pressure_pa"] = 504.0
    case["expected"][AREA_0_30]["verdict"] = "fail"
    assert [d["bucket"] for d in _disagreements(case)] == ["model-gap"]


def test_triage_a_rule_that_links_a_finding_is_a_known_limitation():
    """Label: property - the 0 to 40 degree area links finding #282: a disagreement there is a
    `known-limitation`, and still a false pass."""
    case = _case()
    case["expected"][AREA_0_40]["verdict"] = "fail"
    r = runner.evaluate([case])
    assert [d["bucket"] for d in r["disagreements"]] == ["known-limitation"]
    assert r["per"][AREA_0_40]["false_passes"] == ["box_b"]


def test_triage_oracle_and_engine_agree_against_a_published_verdict_is_a_rule_suspect():
    """Label: property - a published verdict that the oracle and the engine both contradict is a
    `rule-suspect`: the Rule as written may not be what the source applied."""
    case = _case()
    case["provenance"]["verdict_basis"] = "published"
    case["expected"][GM]["verdict"] = "fail"
    case["expected"][GM]["oracle"] = "pass"
    assert [d["bucket"] for d in _disagreements(case)] == ["rule-suspect"]


def test_triage_a_published_verdict_with_no_oracle_verdict_needs_an_expert():
    """Label: property - a published verdict the engine contradicts, with no oracle verdict to break
    the tie, `needs-expert`."""
    case = _case()
    case["provenance"]["verdict_basis"] = "published"
    case["expected"][GM]["verdict"] = "fail"
    assert [d["bucket"] for d in _disagreements(case)] == ["needs-expert"]


def test_results_list_coverage_by_rule_id_and_every_bucket():
    """Label: property - results.md has a row per Rule of each selected set, by rule id, and a count
    per triage bucket."""
    text = (EVAL / "results.md").read_text(encoding="utf-8")
    assert "### is2008-a@1.0.0" in text
    for rule_id in CRITERIA + [WEATHER, "is2008/A.2.2/condition"]:
        assert f"| `{rule_id}` " in text, rule_id
    for bucket in runner.BUCKETS:
        assert f"| {bucket} | " in text, bucket


# --- rule mutation (issue #279) ----------------------------------------------------------------------

import json  # noqa: E402

import rule_mutation  # noqa: E402


def test_every_numeric_limit_of_is2008_a_has_a_mutant_on_each_side():
    """Label: property - each evaluated limit Rule of `is2008-a` gets a mutant one step below and
    one above its limit; a mutant differs from the set in that one number and its set id."""
    text = rule_mutation.set_text("is2008-a@1.0.0")
    base = json.loads(text)
    limits = {r["id"]: r["limit"]["value"] for r in base["rules"]
              if r.get("evaluated", True) and r["kind"] == "limit"}
    ms = rule_mutation.mutants("is2008-a@1.0.0")
    assert sorted((m["rule"], m["side"]) for m in ms) == sorted(
        (r, s) for r in limits for s in ("below", "above"))
    for m in ms:
        doc = json.loads(m["text"])
        assert doc["id"].startswith("is2008-a-mutant-") and doc["family"] == base["family"]
        moved = [r["id"] for r, b in zip(doc["rules"], base["rules"]) if r != b]
        assert moved == [m["rule"]]
        assert m["moved"] == pytest.approx(limits[m["rule"]] + (m["step"] if m["side"] == "above" else -m["step"]))


def test_the_real_corpus_notices_every_mutant_of_is2008_a():
    """Label: property - every limit of `is2008-a`, moved one step either way, changes at least one
    case's verdict: the corpus is not weak anywhere."""
    text = (EVAL / "results.md").read_text(encoding="utf-8")
    # 12 of `is2008-a`, and the mutants of the liquid bulk sets (issue #338): none is weak, but the two of
    # `marpol/23/accidental-oil-outflow` (issue #341): the evaluation harness has no draught, so it cannot work
    # out the mean oil outflow; `ovcd-core/tests/oil_outflow.rs` checks that Rule's method on the published
    # tank barge of Res. MEPC.122(52) instead.
    assert "Not noticed by any case: 2." in text
    section = text.split("## Rule mutation", 1)[1].split("\n## ", 1)[0]
    unnoticed = [ln for ln in section.splitlines() if "none: weak corpus" in ln]
    assert len(unnoticed) == 2 and all("`marpol/23/accidental-oil-outflow`" in ln for ln in unnoticed)
    assert sum(1 for line in section.splitlines() if line.startswith("| `is2008/")) == 12


def test_a_limit_no_case_comes_near_is_a_weak_corpus_warning(tmp_path):
    """Label: property - a fixture set with a GM limit of 5 m, far from every case's GM: no case
    changes its verdict when the limit moves, so both mutants are weak-corpus findings. The GM limit
    of 0.15 m in the same set is noticed."""
    fixture = json.dumps({
        "id": "weak-fixture", "family": "weak", "title": "weak fixture", "version": "1.0.0",
        "edition": {"document": "test"}, "sources": [], "status": "draft",
        "known_conflicts": [{"rules": ["weak/x/gm-high", "weak/x/gm-low"], "reason": "test set"}],
        "rules": [
            {"id": f"weak/x/{slug}", "title": slug, "origin": "physical", "source": {},
             "codified": {"by": "agent", "date": "2026-10-21", "review": "#279"}, "verified": False,
             "kind": "limit", "measure": {"id": "gm0"}, "compare": "ge",
             "limit": {"value": value, "unit": "m"}, "consequence": "block", "evaluated": True,
             "since": "1.0.0"}
            for slug, value in (("gm-low", 0.15), ("gm-high", 5.0))],
    })
    cases = box_cases()
    result = rule_mutation.run(cases, ["weak-fixture@1.0.0"], runner.trace_of, (fixture,))
    weak = sorted((m["rule"], m["side"]) for m in result if not m["noticed_by"])
    assert weak == [("weak/x/gm-high", "above"), ("weak/x/gm-high", "below")]
    # The runner prints a warning annotation, and exits 0, for a weak corpus.
    one = tmp_path / "cases"
    one.mkdir()
    shutil.copy(EVAL / "cases" / "box_a.yaml", one / "box_a.yaml")       # large margins only
    r = run_runner("--cases", str(one), "--mutants", str(tmp_path / "none.yaml"),
                   "--results", str(tmp_path / "results.md"))
    assert r.returncode == 0, r.stdout
    assert r.stdout.count("::warning title=Evaluation weak corpus") == 14
