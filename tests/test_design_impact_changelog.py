"""Where the changelog writer puts an entry (ticket 133)."""
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import design_impact as di  # noqa: E402

LOG = """# Changelog

## Unreleased

### Fixed

- Something.

### Added

- Something else.

## 0.2.2 (2026-10-15)

### Fixed

- Old fix.

## Design changes

Intro text.

### old-branch (2026-10-01)

Between `a` and `b`.

| old |
"""


def write(tmp_path, monkeypatch, text, **kw):
    monkeypatch.setattr(di, "CHANGELOG", tmp_path / "CHANGELOG.md")
    (tmp_path / "CHANGELOG.md").write_text(text, encoding="utf-8")
    di.write_changelog("| T | x |", "branch-x", "main aaa", "this branch bbb")
    return (tmp_path / "CHANGELOG.md").read_text(encoding="utf-8")


def test_entry_goes_under_design_changes_whatever_headings_come_before(tmp_path, monkeypatch):
    out = write(tmp_path, monkeypatch, LOG)
    section = out.index("## Design changes")
    assert out.index("### branch-x") > section
    assert out.index("### branch-x") < out.index("### old-branch")        # newest first
    assert out[:section] == LOG[:LOG.index("## Design changes")]          # nothing above moved
    assert out.count("### Fixed") == 2 and out.count("### Added") == 1


def test_a_second_entry_with_the_same_key_replaces_the_first(tmp_path, monkeypatch):
    out = write(tmp_path, monkeypatch, LOG)
    (tmp_path / "CHANGELOG.md").write_text(out, encoding="utf-8")
    di.write_changelog("| T2 |", "branch-x", "main aaa", "this branch ccc")
    again = (tmp_path / "CHANGELOG.md").read_text(encoding="utf-8")
    assert again.count("### branch-x") == 1 and "| T2 |" in again and "| T | x |" not in again


def test_a_section_after_design_changes_is_kept(tmp_path, monkeypatch):
    out = write(tmp_path, monkeypatch, LOG + "\n## Notes\n\nKeep me.\n")
    assert out.rstrip().endswith("## Notes\n\nKeep me.")
    assert out.index("### branch-x") < out.index("## Notes")


def test_a_changelog_without_the_section_gets_one_at_the_end(tmp_path, monkeypatch):
    out = write(tmp_path, monkeypatch, "# Changelog\n\n## Unreleased\n\n### Fixed\n\n- x\n")
    assert out.index("## Design changes") > out.index("### Fixed")
    assert out.index("### branch-x") > out.index("## Design changes")
    assert out.count("## Design changes") == 1
