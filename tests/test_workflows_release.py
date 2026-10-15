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


# A push (a tag, or main with a version bump) may publish; `tools/release_plan.py` decides.
PUBLISHING = "github.event_name == 'push'"
PLANNED = "needs.verify.outputs.publish == 'true'"


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
    for s in steps:  # nothing that publishes may skip the checks
        if "check-runs" in s.get("run", ""):
            assert s.get("if") == "steps.plan.outputs.publish == 'true'", s


def test_release_tag_check_works_when_called_from_another_workflow():
    """In a called workflow `github.event_name` is the caller's event, so the tag check must key on the ref."""
    text = RELEASE.read_text(encoding="utf-8")
    assert "github.event_name == 'release'" not in text
    assert "startsWith(github.ref, 'refs/tags/v')" in text


def test_release_gate_skips_this_file_in_the_wheel_job():
    assert "--ignore-glob='tests/test_workflows*.py'" in RELEASE.read_text(encoding="utf-8")


def test_app_build_publishes_nothing_to_pypi():
    text = APP_BUILD.read_text(encoding="utf-8").lower()
    for banned in ("pypi-publish", "twine", "maturin publish", "id-token", "environment: pypi"):
        assert banned not in text, banned


# ---------- when a release is made (issue #173) ----------

def test_releases_start_from_a_version_tag_a_test_tag_or_a_version_bump_on_main():
    push = workflow(APP_BUILD)["on"]["push"]
    assert push == {"branches": ["main"], "paths": ["pyproject.toml"], "tags": ["v*", "test-*"]}


def test_verify_plans_first_and_exposes_the_plan():
    verify = workflow(APP_BUILD)["jobs"]["verify"]
    plan = verify["steps"][1]
    assert plan["id"] == "plan" and "tools/release_plan.py" in plan["run"]
    assert plan["env"]["BEFORE_SHA"] == "${{ github.event.before }}"
    assert set(verify["outputs"]) == {"build", "publish", "tag", "kind"}


def test_the_on_main_check_applies_to_numbered_releases_only():
    steps = workflow(APP_BUILD)["jobs"]["verify"]["steps"]
    on_main = next(s for s in steps if "merge-base --is-ancestor" in s.get("run", ""))
    assert on_main["if"] == "steps.plan.outputs.on_main == 'true'"


def test_nothing_builds_or_publishes_unless_the_plan_says_so():
    jobs = workflow(APP_BUILD)["jobs"]
    assert jobs["build"]["if"] == "needs.verify.outputs.build == 'true'"
    assert jobs["gate"]["if"] == PLANNED
    assert jobs["release"]["if"] == PLANNED
    assert "latest" not in jobs, "the rolling latest build was reversed (issue #173)"


def test_the_release_takes_its_tag_and_kind_from_the_plan_through_env():
    job = workflow(APP_BUILD)["jobs"]["release"]
    env = {k: v for s in job["steps"] for k, v in (s.get("env") or {}).items()}
    assert env["TAG"] == "${{ needs.verify.outputs.tag }}"
    assert env["KIND"] == "${{ needs.verify.outputs.kind }}"
    run = "\n".join(s.get("run", "") for s in job["steps"])
    assert "${{" not in run, "plan values reach the shell through env only"
    assert 'gh release create "$TAG"' in run and "--prerelease" in run
    assert '--target "$GITHUB_SHA"' in run and "--verify-tag" in run
    assert "## Unreleased" in run                       # a test release's notes


def test_only_a_version_tag_must_match_the_package_version():
    build = workflow(APP_BUILD)["jobs"]["build"]
    check = next(s for s in build["steps"] if "tomllib" in s.get("run", ""))
    assert check["if"] == "startsWith(github.ref, 'refs/tags/v')"
    assert "startsWith(github.ref, 'refs/tags/v')" in RELEASE.read_text(encoding="utf-8")


def test_a_release_build_is_never_cancelled():
    assert "concurrency" not in workflow(APP_BUILD) or \
        workflow(APP_BUILD)["concurrency"].get("cancel-in-progress") in (False, None)


def test_the_readme_says_how_releases_are_made():
    text = (ROOT / "README.md").read_text(encoding="utf-8")
    assert "releases/tag/latest" not in text
    assert "/releases)" in text
