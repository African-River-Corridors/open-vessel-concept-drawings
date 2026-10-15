"""When the App build workflow releases (issue #173): only on a version bump or a test-release tag.

Tests `tools/release_plan.py`, which the workflow runs first. It reads the repository's own tools, so
the release gate's wheel job skips it with the other `test_workflows*` files.
"""
import os
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import release_plan as rp  # noqa: E402

A, B = "a" * 40, "b" * 40


def versions(**by_commit):
    return lambda commit: by_commit.get(commit)


def run(event, ref, sha=B, before=A, *, at=None, tags=()):
    return rp.plan(event, ref, sha, before, version_at=at or versions(a="0.2.2", b="0.2.2"),
                   tag_exists=lambda t: t in tags)


def test_a_merge_that_keeps_the_version_releases_nothing():
    p = run("push", "refs/heads/main", at=versions(**{A: "0.2.2", B: "0.2.2"}))
    assert (p["build"], p["publish"]) == (False, False)
    assert "unchanged" in p["reason"]


def test_a_merge_that_bumps_the_version_releases_it():
    p = run("push", "refs/heads/main", at=versions(**{A: "0.2.2", B: "0.2.3"}))
    assert (p["build"], p["publish"], p["tag"], p["kind"], p["on_main"]) == (True, True, "v0.2.3", "numbered", True)


def test_a_bump_to_a_version_already_tagged_releases_nothing():
    p = run("push", "refs/heads/main", at=versions(**{A: "0.2.2", B: "0.2.3"}), tags={"v0.2.3"})
    assert (p["build"], p["publish"]) == (False, False)
    assert "exists already" in p["reason"]


def test_a_first_push_with_no_earlier_commit_counts_as_a_bump():
    p = run("push", "refs/heads/main", before="0" * 40, at=versions(**{B: "0.2.3"}))
    assert (p["publish"], p["tag"]) == (True, "v0.2.3")


def test_an_unreadable_version_fails_the_run():
    with pytest.raises(SystemExit, match="cannot read the version"):
        run("push", "refs/heads/main", at=versions())


def test_a_version_tag_makes_a_numbered_release_from_main():
    p = run("push", "refs/tags/v0.3.0")
    assert (p["publish"], p["tag"], p["kind"], p["on_main"]) == (True, "v0.3.0", "numbered", True)


@pytest.mark.parametrize("tag", ["test-release", "test-new-hull-view", "test-2026.10.15"])
def test_a_test_release_tag_makes_a_test_release_from_any_commit(tag):
    p = run("push", f"refs/tags/{tag}")
    assert (p["publish"], p["tag"], p["kind"], p["on_main"]) == (True, tag, "test", False)


@pytest.mark.parametrize("ref", ["refs/tags/release-1", "refs/tags/test-", "refs/tags/test-a b",
                                 "refs/heads/feature", "refs/heads/test-x"])
def test_other_tags_and_branches_release_nothing(ref):
    p = run("push", ref)
    assert (p["build"], p["publish"]) == (False, False)


def test_a_manual_start_builds_but_publishes_nothing():
    p = run("workflow_dispatch", "refs/heads/main")
    assert (p["build"], p["publish"]) == (True, False)


def test_version_at_reads_pyproject_at_a_commit(tmp_path):
    env = {**os.environ, "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.com",
           "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@example.com"}
    g = lambda *a: subprocess.run(["git", *a], cwd=tmp_path, env=env, check=True, capture_output=True, text=True)
    g("init", "-q")
    (tmp_path / "pyproject.toml").write_text('[project]\nname = "x"\nversion = "1.2.3"\n')
    g("add", "pyproject.toml")
    g("commit", "-q", "-m", "one")
    sha = g("rev-parse", "HEAD").stdout.strip()
    old = os.getcwd()
    os.chdir(tmp_path)
    try:
        assert rp.version_at(sha) == "1.2.3"
        assert rp.version_at("0" * 40) is None
    finally:
        os.chdir(old)


def test_main_writes_the_answer_for_the_workflow(tmp_path):
    out = tmp_path / "out"
    r = subprocess.run([sys.executable, str(ROOT / "tools" / "release_plan.py")], capture_output=True, text=True,
                       env={**os.environ, "GITHUB_EVENT_NAME": "push", "GITHUB_REF": "refs/tags/test-x",
                            "GITHUB_OUTPUT": str(out)})
    assert r.returncode == 0, r.stderr
    text = out.read_text()
    assert "publish=true" in text and "tag=test-x" in text and "kind=test" in text and "on_main=false" in text
