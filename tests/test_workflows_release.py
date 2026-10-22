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


# ---------- the wheel check runs only tests it has the files for (issue #211) ----------

def wheel_check_run() -> str:
    steps = workflow(RELEASE)["jobs"]["wheels"]["steps"]
    return next(s["run"] for s in steps if "wheel-check" in s.get("run", ""))


def wheel_check_plan():
    """What the wheel check copies and what it skips, read from release.yml."""
    import shlex
    lines = [ln for ln in wheel_check_run().splitlines() if not ln.strip().startswith("#")]
    words = shlex.split("\n".join(lines).replace("\\\n", " "))
    cp = words.index("cp")
    copied = set(words[cp + 2:words.index('$RUNNER_TEMP/wheel-check/', cp)])
    ignored = [w.split("=", 1)[1] for w in words if w.startswith(("--ignore=", "--ignore-glob="))]
    deselected = [w.split("=", 1)[1] for w in words if w.startswith("--deselect=")]
    return copied, ignored, deselected


def repo_folders_read(path: Path, deselected=()) -> set:
    """The top-level names under the repo root that the tests in a file read, e.g. `ROOT / "tools"`,
    and "checkout" when they import a package module that reads the checkout.

    Starts from module-level code, decorated functions (fixtures) and every test function not in
    `deselected`, and follows the module's own helper functions they name. It sees `ROOT / "x"` (ROOT
    is the repo root by convention here) and `Path(__file__)...parents[n] / "x"` at the repo root."""
    import ast
    import re
    tree = ast.parse(path.read_text(encoding="utf-8"))
    funcs = {n.name: n for n in tree.body if isinstance(n, ast.FunctionDef)}
    todo = [n for n in tree.body if not isinstance(n, ast.FunctionDef)
            or n.decorator_list or (n.name.startswith("test") and n.name not in deselected)]
    seen = []
    while todo:
        node = todo.pop()
        if any(node is s for s in seen):
            continue
        seen.append(node)
        todo += [funcs[n.id] for n in ast.walk(node) if isinstance(n, ast.Name) and n.id in funcs]
    text = "\n".join(ast.unparse(n) for n in seen)
    depth = len(path.relative_to(ROOT).parts) - 1        # parents[depth] of the file is the repo root
    names = set(re.findall(r"\bROOT\s*/\s*['\"]([^'\"/]+)", text))
    names |= set(re.findall(rf"parents\[{depth}\]\s*/\s*['\"]([^'\"/]+)", text))
    checkout = modules_that_read_the_checkout()
    for node in (n for s in seen for n in ast.walk(s)):
        mods = [a.name for a in node.names] if isinstance(node, ast.Import) else []
        if isinstance(node, ast.ImportFrom) and node.module:
            mods = [node.module, *(f"{node.module}.{a.name}" for a in node.names)]
        if checkout & set(mods):
            names.add("checkout")
    if "walk_packages" in text and checkout:             # imports every module of the package
        names.add("checkout")
    return names


def modules_that_read_the_checkout() -> set:
    """Package modules that find files from their own path up to the repo root (`parents[2]`) and read
    one at import. From a wheel that path is in site-packages, so the import fails and a test that
    imports the module can run only in a checkout. A module that reads only when called (issue #217)
    can be imported; its tests give it the folder they need."""
    src = ROOT / "src" / "vessel_designer"
    found = set()
    for p in src.rglob("*.py"):
        text = p.read_text(encoding="utf-8")
        if "parents[2]" in text and reads_files_at_import(text):
            found.add("vessel_designer." + ".".join(p.relative_to(src).with_suffix("").parts))
    return found


def reads_files_at_import(source: str) -> bool:
    """True when module-level code (not a function body, not the `__main__` block) reads a file."""
    import ast
    reads = {"open", "read_text", "read_bytes", "iterdir", "glob", "rglob"}
    todo = list(ast.parse(source).body)
    while todo:
        n = todo.pop()
        if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)):
            continue                                         # runs when called, not at import
        if isinstance(n, ast.If) and "__main__" in ast.unparse(n.test):
            continue
        f = n.func if isinstance(n, ast.Call) else None
        if (isinstance(f, ast.Name) and f.id in reads) or (isinstance(f, ast.Attribute) and f.attr in reads):
            return True
        todo += ast.iter_child_nodes(n)
    return False


def files_under_tests():
    """Python files pytest may load under tests/; tests/data holds data and the scripts that made it."""
    files = sorted(p.relative_to(ROOT).as_posix() for p in (ROOT / "tests").rglob("*.py"))
    assert files
    return [f for f in files if not f.startswith("tests/data/")]


def skipped(f: str, ignored) -> bool:
    from fnmatch import fnmatch
    return any(fnmatch(f, pat) or f == pat for pat in ignored)


def test_the_wheel_check_copies_folders_and_skips_tests_that_exist():
    copied, ignored, deselected = wheel_check_plan()
    assert "tests" in copied and all((ROOT / c).is_dir() for c in copied), copied
    for pat in ignored:
        assert any(skipped(f, [pat]) for f in files_under_tests()), f"{pat} skips nothing"
    for d in deselected:
        f, name = d.split("::")
        assert f"def {name}(" in (ROOT / f).read_text(encoding="utf-8"), d


def test_every_test_the_wheel_check_runs_has_its_files_copied():
    """A test that reads a repo folder the wheel check does not copy (tools/, src/, CHANGELOG.md, ...),
    or imports a package module that reads the checkout, fails the release gate only at release time.
    Catch it here instead: copy the folder in release.yml, or skip the test there (--ignore, --deselect).

    Label: property"""
    copied, ignored, deselected = wheel_check_plan()
    skips = {}
    for d in deselected:
        f, name = d.split("::")
        skips.setdefault(f, set()).add(name)
    missing = {}
    for f in (f for f in files_under_tests() if not skipped(f, ignored)):
        names = repo_folders_read(ROOT / f, skips.get(f, set())) - copied
        if names:
            missing[f] = sorted(names)
    assert missing == {}, f"the wheel check runs these tests but does not copy what they read: {missing}"


def test_the_wheel_check_guard_sees_what_a_test_reads():
    """The guard has teeth: each case below is one that broke, or would have broken, the wheel check.

    Label: property"""
    tests = ROOT / "tests"
    assert "tools" in repo_folders_read(tests / "test_gen_depth_rule.py")
    assert "tools" in repo_folders_read(tests / "test_gen_vessel_classes.py")
    assert "evaluation" in repo_folders_read(tests / "test_evaluation_runner.py")
    # A module that reads from its own path at import cannot import from a wheel (issue #217).
    assert reads_files_at_import('X = (Path(__file__).resolve().parents[2] / "a.yaml").read_text()')
    assert reads_files_at_import('with open(Path(__file__).parents[2] / "a") as f:\n    A = f.read()')
    assert not reads_files_at_import('def a():\n    return (Path(__file__).parents[2] / "a").read_text()')
    assert not reads_files_at_import('if __name__ == "__main__":\n    open("a").read()')
    assert not reads_files_at_import('class A:\n    def a(self):\n        return open("a").read()')
    # The calibration module reads its folder only when used, so tests that import it run from a wheel.
    assert "vessel_designer.calibration" not in modules_that_read_the_checkout()
    assert "checkout" not in repo_folders_read(tests / "test_calibration_held_out.py")
    # The package walk sits in a helper; it counts while a test that calls the helper runs.
    assert "CHANGELOG.md" in repo_folders_read(tests / "test_public_api.py")
    _, _, deselected = wheel_check_plan()
    api = {d.split("::")[1] for d in deselected if d.startswith("tests/test_public_api.py::")}
    assert repo_folders_read(tests / "test_public_api.py", api) == set()
    assert repo_folders_read(tests / "core" / "test_barge_cost.py") == {"examples"}


# ---------- publishing to PyPI (issue #1) ----------

PUBLISH = ROOT / ".github" / "workflows" / "publish-pypi.yml"


def test_pypi_publishing_starts_only_from_a_version_tag():
    """Label: property
    """
    wf = workflow(PUBLISH)
    assert wf["on"] == {"push": {"tags": ["v*"]}}


def test_pypi_publishing_waits_for_the_release_gate_and_the_sdist():
    """Label: property
    """
    jobs = workflow(PUBLISH)["jobs"]
    assert jobs["gate"]["uses"] == "./.github/workflows/release.yml"
    assert needs(jobs["sdist"]) == {"gate"}
    assert {"gate", "sdist"} <= needs(jobs["publish"])


def test_the_pypi_job_uses_trusted_publishing_with_no_token():
    """Label: property
    """
    wf = workflow(PUBLISH)
    text = PUBLISH.read_text(encoding="utf-8")
    pub = wf["jobs"]["publish"]
    assert pub["environment"] == "pypi"
    assert pub["permissions"] == {"id-token": "write"}
    assert wf["permissions"] == {"contents": "read"}  # the other jobs get no id-token
    for banned in ("secrets.", "password", "api-token", "twine", "maturin publish"):
        assert banned not in text.lower(), banned
    uses = [s["uses"] for s in pub["steps"] if "pypi-publish" in s.get("uses", "")]
    assert len(uses) == 1 and uses[0].split("@")[1].isalnum() and len(uses[0].split("@")[1]) == 40


def test_the_pypi_job_takes_the_wheels_the_gate_tested():
    """Label: property
    """
    steps = workflow(PUBLISH)["jobs"]["publish"]["steps"]
    pattern = next(s["with"]["pattern"] for s in steps if "download-artifact" in s.get("uses", ""))
    assert "wheel-*" in pattern and "sdist" in pattern
