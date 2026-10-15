"""The design-impact check (ticket 28): what the tool reports, and that it can fail.

The tool builds `ovcd` twice and runs a fixed set of Designs. These tests use a stand-in `ovcd`
(a small script) so they need no compiler: they prove the report, the table, the changelog entry
and the fail-closed checks. The real binaries run in the gate and in the pull-request workflow.
"""
import json
import stat
import sys
import textwrap
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import design_impact as di  # noqa: E402


def fake_ovcd(tmp_path: Path, name: str, factor: float = 1.0, fail_on: str = "") -> Path:
    """A stand-in `ovcd`: `fill` copies the file, `run --json` answers from the inputs."""
    script = tmp_path / name
    script.write_text(textwrap.dedent(f'''\
        #!{sys.executable}
        import json, shutil, sys
        cmd = sys.argv[1]
        if cmd == "fill":
            text = open(sys.argv[2]).read()
            if {fail_on!r} and {fail_on!r} in text:
                print("error: refused", file=sys.stderr); sys.exit(1)
            open(sys.argv[3], "w").write(text)
        elif cmd == "run":
            d = json.load(open(sys.argv[3]))
            o = d["options"][0]
            loa = o["inputs"]["length_overall_m"]
            print(json.dumps({{"options": [{{"name": o["name"], "results": {{
                "results": {{"cargo_t": loa * 10 * {factor}, "governs": "hold volume"}},
                "checks": [{{"name": "Cargo > 0", "ok": True}}]}}}}]}}))
    '''))
    script.chmod(script.stat().st_mode | stat.S_IEXEC)
    return script


def test_the_case_set_covers_the_calibration_vessels_and_a_sweep():
    cs = di.cases()
    assert len(cs) >= 25
    assert any(k.startswith("calibration para-mississippi") for k in cs)
    assert any(k.startswith("sweep CN-derived liquid") for k in cs)
    assert any(k.startswith("sweep gas IGC_2G_EXACT") for k in cs)
    assert not any("ADN_G" in k for k in cs)
    for typed in cs.values():           # the five required inputs, so `ovcd fill` can complete the rest
        for key in ("cargo", "hull_form", "formation", "length_overall_m", "beam_m"):
            assert key in typed
    # Most cases type their depth, so a design that types it is compared as before; the cases that
    # leave it out are the default-depth rule's (issue #120).
    left_out = [k for k, v in cs.items() if "moulded_depth_m" not in v]
    assert sorted(left_out) == ["sweep default depth dry bulk 90x14", "sweep default depth gas 90x14"]


def test_identical_results_say_no_design_change(tmp_path):
    cs = {"a": di._inputs("bauxite", "box_hold", "pushed_barge", 60.0, 10.0, 4.0)}
    base = di.run_all(fake_ovcd(tmp_path, "base"), cs)
    head = di.run_all(fake_ovcd(tmp_path, "head"), cs)
    rows = di.compare(base, head)
    assert rows == []
    text, tbl = di.report_text(len(cs), rows, "main abc", "this branch def")
    assert "no design change" in text and tbl == ""


def test_a_moved_coefficient_gives_a_before_after_table(tmp_path):
    cs = {"a": di._inputs("bauxite", "box_hold", "pushed_barge", 60.0, 10.0, 4.0),
          "b": di._inputs("bauxite", "box_hold", "pushed_barge", 70.0, 10.0, 4.0)}
    base = di.run_all(fake_ovcd(tmp_path, "base"), cs)
    head = di.run_all(fake_ovcd(tmp_path, "head", factor=1.1), cs)
    rows = di.compare(base, head)
    assert {r["case"] for r in rows} == {"a", "b"}
    row = next(r for r in rows if r["case"] == "a")
    assert row["quantity"] == "results.cargo_t"
    assert row["before"] == 600.0 and row["after"] == pytest.approx(660.0)
    assert row["change"] == "+10 %"
    text, tbl = di.report_text(len(cs), rows, "main abc", "this branch def")
    assert "**design change**: 2 of 2" in text and "design-change" in text
    assert "| a | `results.cargo_t` | 600 | 660 | +10 % |" in tbl
    assert "| Case | Quantity | Before | After | Change |" in tbl


def test_noise_below_the_tolerance_is_not_a_change():
    assert di.same(1.0, 1.0 + 1e-12) and not di.same(1.0, 1.0 + 1e-6)
    assert not di.same(True, 1) and not di.same(1, True)
    before = {"a": {"results": {"x": 100.0, "y": 5.0, "name": "k"}}}
    after = {"a": {"results": {"x": 100.0 * (1 + 1e-12), "y": 6.0, "name": "k"}}}
    rows = di.compare(before, after)
    assert [r["quantity"] for r in rows] == ["y"]


def test_the_biggest_moves_come_first():
    before = {"a": {"results": {"p": 100.0, "q": 100.0, "r": "old"}}}
    after = {"a": {"results": {"p": 101.0, "q": 150.0, "r": "new"}}}
    assert [r["quantity"] for r in di.compare(before, after)] == ["r", "q", "p"]


def test_a_case_that_stops_computing_is_a_change(tmp_path):
    cs = {"a": di._inputs("bauxite", "box_hold", "pushed_barge", 60.0, 10.0, 4.0)}
    base = di.run_all(fake_ovcd(tmp_path, "base"), cs)
    head = di.run_all(fake_ovcd(tmp_path, "head", fail_on='"length_overall_m": 60.0'), cs)
    rows = di.compare(base, head)
    assert len(rows) == 1 and rows[0]["quantity"] == "result"
    assert rows[0]["before"] == "computed" and "refused" in rows[0]["after"]


def test_a_long_table_is_cut_and_says_how_much_was_left_out():
    rows = [{"case": f"c{i}", "quantity": "q", "before": 1.0, "after": 2.0, "change": "+100 %"}
            for i in range(di.MAX_ROWS + 7)]
    assert "7 more changed quantities" in di.table(rows)


def test_table_cells_cannot_break_the_markdown():
    rows = [{"case": "a|b", "quantity": "q", "before": "x|y", "after": "z\nw", "change": "changed"}]
    line = di.table(rows).splitlines()[2]
    assert line.count("|") - line.count("\\|") == 6


def test_the_changelog_entry_is_written_once_per_key_and_newest_first(tmp_path, monkeypatch):
    monkeypatch.setattr(di, "CHANGELOG", tmp_path / "CHANGELOG.md")
    di.write_changelog("| t1 |", "branch-one", "main aaa", "this branch bbb")
    di.write_changelog("| t2 |", "branch-two", "main aaa", "this branch ccc")
    di.write_changelog("| t1b |", "branch-one", "main aaa", "this branch ddd")   # replaces its entry
    text = (tmp_path / "CHANGELOG.md").read_text()
    assert text.startswith("# Changelog") and "## Design changes" in text
    assert text.count("### branch-one") == 1 and text.count("### branch-two") == 1
    assert "| t1b |" in text and "| t1 |" not in text
    assert text.index("### branch-one") < text.index("### branch-two")   # the replaced one is newest
    assert di.changelog_has("| t2 |") and not di.changelog_has("| nope |")


def test_the_gate_fails_closed_when_nothing_computed(tmp_path):
    cs = {"a": di._inputs("bauxite", "box_hold", "pushed_barge", 60.0, 10.0, 4.0)}
    out = di.run_all(fake_ovcd(tmp_path, "dead", fail_on="bauxite"), cs)
    assert not any("results" in v for v in out.values())


def test_publish_replaces_its_own_block_in_the_pr_body(tmp_path, monkeypatch):
    calls = []

    def fake_run(cmd, **kw):
        calls.append(cmd)
        class R:
            returncode = 0
            stdout = json.dumps({"body": f"Closes #28\n\n{di.START}\nold\n{di.END}\n\nmore text"})
        return R()

    monkeypatch.setattr(di.subprocess, "run", fake_run)
    monkeypatch.setattr(di, "OUT", tmp_path)
    di.publish("## Design impact\n\nno design change", False, 7)
    body = (tmp_path / "body.md").read_text()
    assert body.count(di.START) == 1 and "old" not in body
    assert "no design change" in body and "Closes #28" in body and "more text" in body
    assert any(c[:3] == ["gh", "pr", "edit"] and "--remove-label" in c for c in calls)
    calls.clear()
    di.publish("## Design impact\n\n**design change**", True, 7)
    assert any("--add-label" in c and di.LABEL in c for c in calls)


def test_the_fast_gate_runs_the_check_and_counts_it():
    gates = (ROOT / "tools" / "gates.sh").read_text(encoding="utf-8")
    assert 'step "design impact" "$PY" tools/design_impact.py gate' in gates
    steps = [line for line in gates.splitlines() if line.startswith("step ")]
    expected = next(line for line in gates.splitlines() if line.startswith("EXPECTED_STEPS="))
    assert expected == f"EXPECTED_STEPS={len(steps)}"


def test_the_pull_request_workflow_only_measures_and_keeps_an_artifact():
    text = (ROOT / ".github" / "workflows" / "design-impact.yml").read_text(encoding="utf-8")
    assert "design_impact.py report" in text and "--publish" not in text
    assert "pull-requests: write" not in text and "issues: write" not in text   # runs fork code: read only
    assert "pull_request_target" not in text
    assert "upload-artifact" in text and "pr.txt" in text


def versioned_ovcd(tmp_path: Path, name: str, reads: tuple) -> Path:
    """A stand-in `ovcd` that reads only the format versions in `reads`, as the real one says."""
    script = tmp_path / name
    script.write_text(textwrap.dedent(f'''\
        #!{sys.executable}
        import json, sys
        cmd = sys.argv[1]
        if cmd == "fill":
            text = open(sys.argv[2]).read()
            v = json.loads(text)["format_version"]
            if v not in {reads!r}:
                print(f"error: format_version {{v}} is not supported", file=sys.stderr); sys.exit(1)
            open(sys.argv[3], "w").write(text)
        elif cmd == "run":
            d = json.load(open(sys.argv[3]))
            print(json.dumps({{"options": [{{"name": "a", "results": {{
                "results": {{"format_version_read": d["format_version"]}}}}}}]}}))
    '''))
    script.chmod(script.stat().st_mode | stat.S_IEXEC)
    return script


def test_a_case_is_given_to_each_binary_in_the_newest_format_it_reads(tmp_path):
    """Version 2 for a binary that reads it (the head), version 1 for one that does not (a base
    from before issue #175). Version 1 given to the head would give a gas design the 12 m tank
    diameter cap of version 1 files, and hide the change of the default.

    Label: property
    """
    cs = {"a": di._inputs("bauxite", "box_hold", "pushed_barge", 60.0, 10.0, 4.0)}
    new = di.run_all(versioned_ovcd(tmp_path, "new", (1, 2)), cs)
    old = di.run_all(versioned_ovcd(tmp_path, "old", (1,)), cs)
    assert new["a"]["results"]["results"]["format_version_read"] == 2
    assert old["a"]["results"]["results"]["format_version_read"] == 1
    none = di.run_all(versioned_ovcd(tmp_path, "none", ()), cs)
    assert "format_version" in none["a"]["error"]
