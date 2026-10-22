"""The fast gate is split across parallel CI jobs (issue #561). The job named `test` must stand for all
of them: a group of steps that no job runs, or a part that `test` does not wait for, would let a red
check merge.

Label: property
"""
import re
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
CI = yaml.safe_load((ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8"))
GATES = (ROOT / "tools" / "gates.sh").read_text(encoding="utf-8")


def _gate_jobs():
    return {n: j for n, j in CI["jobs"].items() if n.startswith("gate-")}


def _env_of(job):
    out = {}
    for step in job["steps"]:
        out.update(step.get("env", {}))
    return out


def test_test_waits_for_every_gate_job_and_does_not_skip_on_failure():
    job = CI["jobs"]["test"]
    assert set(job["needs"]) == set(_gate_jobs())
    assert job["if"] == "always()"        # a failed part must fail `test`, not skip it
    text = "\n".join(s.get("run", "") for s in job["steps"])
    for name in _gate_jobs():
        assert f'"{name}"' in text, name


def test_the_gate_jobs_together_run_every_step_group_of_the_script():
    groups = re.search(r'GROUPS_ALL="([^"]*)"', GATES).group(1).split()
    run = set()
    for job in _gate_jobs().values():
        run |= set(filter(None, _env_of(job).get("GATES_STEPS", "").split(",")))
    assert run == set(groups), (run, groups)


def test_the_rust_tests_run_in_every_partition_once():
    job = CI["jobs"]["gate-rust"]
    parts = job["strategy"]["matrix"]["part"]
    assert parts == list(range(1, len(parts) + 1))
    assert _env_of(job)["GATES_PARTITION"] == "${{ matrix.part }}/" + str(len(parts))
    assert "fail-fast" in job["strategy"] and job["strategy"]["fail-fast"] is False


def test_every_step_of_the_script_names_a_group():
    steps = [l.split()[1] for l in GATES.splitlines() if l.startswith("step ")]
    groups = set(re.search(r'GROUPS_ALL="([^"]*)"', GATES).group(1).split())
    assert len(steps) == 9 and set(steps) == groups


def test_the_required_checks_also_run_for_a_merge_group():
    on = CI.get("on", CI.get(True))        # YAML reads a bare `on` as the boolean True
    assert "merge_group" in on
    assert "test" in CI["jobs"] and "release-guard" in CI["jobs"]
    # A merge group is never cancelled by a newer one: its concurrency group is its own run id.
    group = CI["concurrency"]["group"]
    assert "github.run_id" in group and "event_name == 'pull_request'" in group
