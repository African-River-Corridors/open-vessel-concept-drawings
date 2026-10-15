"""Release publishing guard (ticket 129): nothing is built or published for a tag unless `test` and
`release-guard` passed on a commit that is on main, and the release gate ran. Reads app-build.yml."""
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
APP_BUILD = ROOT / ".github" / "workflows" / "app-build.yml"
RELEASE = ROOT / ".github" / "workflows" / "release.yml"


def workflow(path: Path) -> dict:
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    data["on"] = data.pop(True, data.get("on"))  # YAML reads a bare `on` as True
    return data


def needs(job: dict) -> set:
    n = job.get("needs", [])
    return {n} if isinstance(n, str) else set(n)


def test_the_publishing_job_waits_for_verify_and_the_release_gate():
    jobs = workflow(APP_BUILD)["jobs"]
    publishing = [n for n, j in jobs.items() if "gh release create" in str(j)]
    assert publishing, "app-build.yml no longer publishes a release?"
    gate = [n for n, j in jobs.items() if str(j.get("uses", "")).endswith("release.yml")]
    assert gate, "app-build.yml must call release.yml (wheels, binaries, pytest) before publishing"
    for name in publishing:
        assert {"verify", *gate} <= needs(jobs[name]), name
        assert "always()" not in str(jobs[name].get("if", "")), f"{name} can run past a failed gate"


def test_every_job_that_builds_waits_for_verify():
    jobs = workflow(APP_BUILD)["jobs"]
    assert "verify" in needs(jobs["build"])
    assert "always()" not in str(jobs["build"].get("if", ""))


def test_release_workflow_can_be_called():
    assert "workflow_call" in workflow(RELEASE)["on"]


def test_verify_checks_main_and_both_ci_jobs_on_the_tagged_commit():
    verify = workflow(APP_BUILD)["jobs"]["verify"]
    text = str(verify)
    assert "merge-base --is-ancestor" in text and "origin/main" in text
    assert "check-runs" in text and "GITHUB_SHA" in text
    assert "test release-guard" in text
    assert verify["permissions"].get("checks") == "read"


def test_verify_fails_closed_on_a_failed_or_missing_check():
    steps = workflow(APP_BUILD)["jobs"]["verify"]["steps"]
    run = next(s["run"] for s in steps if "check-runs" in s.get("run", ""))
    assert "exit 1" in run
    assert "did not succeed" in run and "timed out" in run
    for s in steps:  # a tag must not skip the checks
        assert "refs/tags/" in s.get("if", ""), s


def test_release_tag_check_works_when_called_from_another_workflow():
    """In a called workflow `github.event_name` is the caller's event, so the tag check must key on the ref."""
    text = RELEASE.read_text(encoding="utf-8")
    assert "github.event_name == 'release'" not in text
    assert "startsWith(github.ref, 'refs/tags/')" in text


def test_release_gate_skips_this_file_in_the_wheel_job():
    assert "--ignore-glob='tests/test_workflows*.py'" in RELEASE.read_text(encoding="utf-8")


def test_app_build_publishes_nothing_to_pypi():
    text = APP_BUILD.read_text(encoding="utf-8").lower()
    for banned in ("pypi-publish", "twine", "maturin publish", "id-token", "environment: pypi"):
        assert banned not in text, banned
