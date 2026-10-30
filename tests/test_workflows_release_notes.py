"""Release notes fit a GitHub release (`tools/release_notes.py`). The 0.3.0 publish failed with HTTP 422:
its CHANGELOG section was 139,695 characters, and a release body holds at most 125,000.

It reads the repository's own tools and CHANGELOG.md, so the release gate's wheel job skips it with the
other `test_workflows*` files.
"""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import release_notes as rn  # noqa: E402

LINK = "https://example.org/CHANGELOG.md"

# The footer app-build.yml adds after the cut (two short lines).
FOOTER = 400


def section(version: str) -> str:
    """The section of CHANGELOG.md the workflow's awk takes for `version`."""
    out, on = [], False
    for line in (ROOT / "CHANGELOG.md").read_text(encoding="utf-8").splitlines(keepends=True):
        if line.startswith("## "):
            if on:
                break
            if line.startswith(f"## {version}"):
                on = True
                continue
        if on:
            out.append(line)
    return "".join(out)


def test_notes_that_fit_are_unchanged():
    text = "- one\n- two\n"
    assert rn.cap(text, LINK, limit=100) == text


def test_long_notes_are_cut_at_a_line_and_end_with_the_link():
    lines = [f"- change {i}: " + "x" * 50 + "\n" for i in range(100)]
    out = rn.cap("".join(lines), LINK, limit=2_000)
    assert len(out) <= 2_000
    kept, tail = out.split("\n\nThe list is cut here")
    assert kept.startswith("- change 0:")
    assert all(line + "\n" in lines for line in kept.splitlines()), "a line was cut through"
    assert LINK in tail


def test_the_0_3_0_notes_fit_and_keep_the_rename_text_and_the_calibration_cases():
    """Label: regression"""
    text = section("0.3.0")
    assert len(text) > rn.LIMIT, "the 0.3.0 section is the case that failed; it should still need a cut"
    out = rn.cap(text, LINK)
    assert len(out) + FOOTER <= rn.GITHUB_LIMIT
    assert "is now Open Vessel Designer (OVD). The repository, crates" in out
    assert "`wilcarry-1750-45x20`" in out  # the last calibration case named
    assert out.endswith(f"[CHANGELOG.md]({LINK}).\n")


def test_the_workflow_cuts_the_notes_before_it_adds_the_footer():
    run = (ROOT / ".github" / "workflows" / "app-build.yml").read_text(encoding="utf-8")
    cut = run.index("python3 tools/release_notes.py notes.md")
    assert run.index("> notes.md", 0, cut) < cut < run.index("These installers are not signed")
    assert "blob/$GITHUB_SHA/CHANGELOG.md" in run
