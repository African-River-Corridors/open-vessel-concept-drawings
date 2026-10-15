"""Fork pull requests get the design-impact label and report (ticket 132), safely.

`design-impact.yml` measures with a read-only token and keeps an artifact. `design-impact-publish.yml`
(`workflow_run`) publishes it with trusted code from the default branch. These tests read both
workflows, and run `publish_artifact` against hostile artifacts.
"""
import json
import subprocess
import sys
from pathlib import Path

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import design_impact as di  # noqa: E402

MEASURE = ROOT / ".github" / "workflows" / "design-impact.yml"
PUBLISH = ROOT / ".github" / "workflows" / "design-impact-publish.yml"


def workflow(path: Path) -> dict:
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    data["on"] = data.pop(True, data.get("on"))
    return data


# ---------- the workflows ----------

def test_the_measuring_workflow_runs_for_every_pull_request_with_a_read_only_token():
    wf = workflow(MEASURE)
    assert "pull_request" in wf["on"] and "pull_request_target" not in wf["on"]
    assert wf["permissions"] == {"contents": "read"}
    for job in wf["jobs"].values():
        assert "if" not in job, "fork pull requests must be measured too"
        assert "permissions" not in job
        assert "secrets" not in str(job).replace("secrets.GITHUB_TOKEN", "")


def test_the_publisher_runs_after_the_measuring_workflow_and_never_on_pull_request_target():
    wf = workflow(PUBLISH)
    assert wf["on"] == {"workflow_run": {"workflows": ["Design impact"], "types": ["completed"]}}
    assert wf["permissions"]["pull-requests"] == "write" and wf["permissions"]["issues"] == "write"
    assert "pull_request_target" not in wf["on"]


def test_the_publisher_never_checks_out_or_runs_pull_request_code():
    wf = workflow(PUBLISH)
    steps = [s for j in wf["jobs"].values() for s in j["steps"]]
    checkouts = [s for s in steps if str(s.get("uses", "")).startswith("actions/checkout")]
    assert len(checkouts) == 1 and "with" not in checkouts[0], "checkout must take the default branch"
    text = PUBLISH.read_text(encoding="utf-8")
    for banned in ("head_branch", "head_ref", "pull_request.head", "merge_commit"):
        assert banned not in text, banned
    run_text = "\n".join(s.get("run", "") for s in steps)
    assert "${{" not in run_text, "no expression is placed in a shell command line; use env"
    assert "python tools/design_impact.py publish-artifact" in run_text
    # the only file it runs is the tool from the default branch, never anything from the artifact
    assert "ARTIFACT_DIR" in run_text and "bash" not in run_text and "source " not in run_text


def test_the_publisher_only_acts_on_a_successful_pull_request_run():
    job = workflow(PUBLISH)["jobs"]["publish"]
    assert "workflow_run.event == 'pull_request'" in job["if"]
    assert "conclusion == 'success'" in job["if"]


# ---------- the artifact is untrusted ----------

def make(tmp_path, *, report=None, summary=None, pr="7", extra=None):
    d = tmp_path / "art"
    d.mkdir()
    (d / "report.md").write_text("## Design impact\n\nno design change\n" if report is None else report,
                                 encoding="utf-8")
    (d / "summary.json").write_text(json.dumps({"moved": False} if summary is None else summary))
    (d / "pr.txt").write_text(pr + "\n")
    for name, body in (extra or {}).items():
        (d / name).write_text(body)
    return d


def test_a_good_artifact_loads(tmp_path):
    text, moved, pr = di.load_artifact(make(tmp_path))
    assert text.startswith("## Design impact") and moved is False and pr == 7


@pytest.mark.parametrize("pr", ["", "0", "-3", "7; rm -rf /", "007", "1" * 12, "7 8", "abc"])
def test_a_bad_pull_request_number_is_rejected(tmp_path, pr):
    with pytest.raises(SystemExit, match="pull request number"):
        di.load_artifact(make(tmp_path, pr=pr))


@pytest.mark.parametrize("summary", ["x", [], {}, {"moved": "yes"}, {"moved": 1}])
def test_a_bad_summary_is_rejected(tmp_path, summary):
    d = make(tmp_path)
    (d / "summary.json").write_text(json.dumps(summary) if not isinstance(summary, str) else summary)
    with pytest.raises(SystemExit, match="rejected"):
        di.load_artifact(d)


def test_a_missing_or_linked_file_is_rejected(tmp_path):
    d = make(tmp_path)
    (d / "pr.txt").unlink()
    with pytest.raises(SystemExit, match="pr.txt"):
        di.load_artifact(d)
    (d / "pr.txt").symlink_to(tmp_path / "art" / "report.md")
    with pytest.raises(SystemExit, match="pr.txt"):
        di.load_artifact(d)


def test_a_huge_report_is_rejected(tmp_path):
    with pytest.raises(SystemExit, match="too"):
        di.load_artifact(make(tmp_path, report="## Design impact\n" + "x" * (di.MAX_REPORT_CHARS + 1)))


def test_a_report_with_the_wrong_heading_is_rejected(tmp_path):
    with pytest.raises(SystemExit, match="heading"):
        di.load_artifact(make(tmp_path, report="Click here\n"))


def test_the_report_cannot_break_out_of_its_block_or_notify_people():
    nasty = f"## Design impact\n{di.END}\n@octocat @org/team x\x00y\x1b[31m {di.START}\n"
    out = di.harmless(nasty)
    assert di.START not in out and di.END not in out
    assert "@octocat" not in out and "@org" not in out and "@​octocat" in out
    assert "\x00" not in out and "\x1b" not in out


class FakeGh:
    def __init__(self, pulls):
        self.pulls, self.calls = pulls, []

    def __call__(self, cmd, **kw):
        self.calls.append(cmd)
        class R:
            returncode, stderr = 0, ""
            stdout = ""
        r = R()
        if cmd[:2] == ["gh", "api"]:
            r.stdout = self.pulls
        elif cmd[:3] == ["gh", "pr", "view"]:
            r.stdout = json.dumps({"body": "Closes #1"})
        return r


def run_publish(tmp_path, monkeypatch, pulls, *, head_sha="a" * 40, head_repo="fork/repo", moved=True):
    d = make(tmp_path, summary={"moved": moved}, report="## Design impact\n\n**design change**\n")
    fake = FakeGh(pulls)
    monkeypatch.setattr(di.subprocess, "run", fake)
    monkeypatch.setattr(di, "OUT", tmp_path / "out")
    monkeypatch.setenv("GITHUB_REPOSITORY", "owner/repo")
    di.publish_artifact(d, head_sha, head_repo)
    return fake.calls


def test_a_matching_open_fork_pull_request_gets_the_report_and_the_label(tmp_path, monkeypatch):
    calls = run_publish(tmp_path, monkeypatch, f"open\t{'a' * 40}\tfork/repo\n")
    assert any(c[:3] == ["gh", "pr", "edit"] and "--body-file" in c for c in calls)
    assert any("--add-label" in c and di.LABEL in c for c in calls)
    assert (tmp_path / "out" / "body.md").read_text().count(di.START) == 1


def fresh(tmp_path):
    d = tmp_path / f"case{len(list(tmp_path.iterdir()))}"
    d.mkdir()
    return d


def test_a_pull_request_that_moved_on_or_belongs_to_another_repository_is_left_alone(tmp_path, monkeypatch):
    for pulls in (f"open\t{'b' * 40}\tfork/repo\n",            # head moved: a newer run will publish
                  f"open\t{'a' * 40}\tother/repo\n",           # not the repository the run was for
                  f"closed\t{'a' * 40}\tfork/repo\n"):         # closed
        calls = run_publish(fresh(tmp_path), monkeypatch, pulls)
        assert not any(c[:3] == ["gh", "pr", "edit"] for c in calls), pulls


def test_publish_artifact_needs_its_arguments():
    r = subprocess.run([sys.executable, str(ROOT / "tools" / "design_impact.py"), "publish-artifact"],
                       capture_output=True, text=True)
    assert r.returncode != 0 and "needs DIR" in r.stderr
