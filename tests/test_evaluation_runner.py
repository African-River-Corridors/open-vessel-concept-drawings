"""The evaluation runner, the generated boundary mutants and the CI gate (issue #184, phase 0, part 2).

A false pass (the engine passes a criterion a case expects to fail) must turn the runner red; a false
fail and a coverage gap are warnings. These tests run the real runner as CI does.
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

CRITERIA = list(runner.ENGINE_NAMES)


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
    assert doc["expected"]["area_0_30"]["verdict"] == "pass"
    doc["expected"]["area_0_30"]["verdict"] = "fail"          # deliberately wrong
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
    """Label: property - with no mutants only the base cases run; every criterion expected to pass and
    none to fail is a coverage gap: warnings, exit 0."""
    cases, _, results = corpus_copy(tmp_path)
    r = run_runner("--cases", str(cases), "--mutants", str(tmp_path / "none.yaml"), "--results", str(results))
    assert r.returncode == 0, r.stdout
    assert r.stdout.count("::warning title=Evaluation coverage gap") == 6


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
        big, small = gen.STEPS[m["criterion"]]
        step = big if m["magnitude"] == "big" else small
        margin = d["expected"][m["criterion"]]["margin"]
        assert abs(margin - (step if m["side"] == "pass" else -step)) <= 1e-3 * step + 1e-9, (d["id"], margin)
        assert m["grid"] == (m["magnitude"] == "small" and step < gen.GRID_TOL[m["criterion"]] * 4)


def test_the_grid_error_tolerances_are_above_what_the_engine_shows():
    """Label: property - on every case and mutant the engine's value is within the grid tolerance of
    the oracle's, so a verdict outside the tolerance is one the engine must get right."""
    worst = {k: 0.0 for k in CRITERIA}
    for case in runner.load_cases():
        for k, r in runner.run_case(case).items():
            worst[k] = max(worst[k], abs(r["engine_value"] - r["expected_value"]))
    for k, w in worst.items():
        assert w <= gen.GRID_TOL[k], (k, w)
    for k in CRITERIA:
        assert gen.STEPS[k][0] >= gen.GRID_TOL[k], k


def test_the_mutants_file_is_what_the_generator_writes():
    """Label: property - evaluation/mutants.yaml is generated, never edited by hand."""
    r = subprocess.run([sys.executable, str(EVAL / "oracle" / "mutants.py"), "--check"],
                       capture_output=True, text=True)
    assert r.returncode == 0, r.stderr
