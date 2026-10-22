"""Changelog fragments (issue #560): one `changes/<branch>.md` per pull request.

Label: property. Each test states a rule of the tooling; no engineering number is pinned.
"""
import importlib.util
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import changelog_fragments as cf  # noqa: E402

spec = importlib.util.spec_from_file_location("di_frag", ROOT / "tools" / "design_impact.py")
di = importlib.util.module_from_spec(spec)
spec.loader.exec_module(di)

TABLE = "| Case | Quantity | Before | After | Change |\n|---|---|---|---|---|\n| a | `x` | 1 | 2 | +100 % |"
IMPACT = f"### ticket/1-x (2026-10-22)\n\nBetween `main abc1234` and `this branch def5678`.\n\n{TABLE}"
GOOD = f"{cf.UNRELEASED}\n- Change (issue #1): a bullet.\n\n{cf.IMPACT}\n{IMPACT}\n"
LOG = "# Changelog\n\n## Unreleased\n\n- old bullet\n\n## 0.2.4 (2026-10-21)\n\n- old note\n\n" \
      "## Design changes\n\nIntro.\n\n### old/entry (2026-10-20)\n\nold table\n"


def test_a_branch_name_gives_a_flat_file_name():
    assert cf.fragment_name("ticket/449-bilge-sheer-camber") == "ticket-449-bilge-sheer-camber.md"
    assert cf.fragment_name("branch abc") == "branch-abc.md"
    assert cf.fragment_name("///") == "change.md"


def test_a_good_fragment_is_well_formed_and_bad_ones_name_their_fault():
    assert cf.problems(GOOD) == []
    assert cf.problems(f"{cf.UNRELEASED}\n- only bullets\n") == []
    assert cf.problems("") != []
    assert any("before the first marker" in p for p in cf.problems("loose text\n" + GOOD))
    assert any("not a bullet" in p for p in cf.problems(f"{cf.UNRELEASED}\nno dash here\n"))
    assert any("no table" in p for p in cf.problems(f"{cf.IMPACT}\n### k (2026-10-22)\n\nno table\n"))
    assert any("heading" in p for p in cf.problems(f"{cf.IMPACT}\n{TABLE}\n"))
    assert any("more than once" in p for p in cf.problems(GOOD + GOOD))


def test_collecting_adds_the_text_and_changes_nothing_else():
    out = cf.collect_text(LOG, [GOOD])
    assert out.count("- Change (issue #1): a bullet.") == 1
    assert out.count("### ticket/1-x (2026-10-22)") == 1
    # Take the added text out again: the old file comes back byte for byte.
    back = out.replace("- Change (issue #1): a bullet.\n", "", 1)
    back = back.replace("\n" + IMPACT + "\n", "", 1)
    assert back == LOG


def test_the_real_changelog_keeps_every_byte_when_fragments_are_collected():
    text = (ROOT / "CHANGELOG.md").read_text(encoding="utf-8")
    out = cf.collect_text(text, [GOOD])
    back = out.replace("- Change (issue #1): a bullet.\n", "", 1).replace("\n" + IMPACT + "\n", "", 1)
    assert back == text


def test_a_design_entry_goes_to_the_top_of_the_design_changes_section():
    out = cf.collect_text(LOG, [GOOD])
    assert out.index("### ticket/1-x") < out.index("### old/entry")
    assert out.index("- Change (issue #1)") < out.index("- old bullet")


def test_writing_an_entry_keeps_the_branchs_own_bullets(tmp_path):
    cf.write_impact("ticket/9-y", IMPACT, tmp_path)
    path = cf.fragment_path("ticket/9-y", tmp_path)
    path.write_text(cf.join("- my bullet", IMPACT), encoding="utf-8")
    cf.write_impact("ticket/9-y", IMPACT.replace("+100 %", "+200 %"), tmp_path)
    unreleased, impact = cf.split(path.read_text(encoding="utf-8"))
    assert unreleased == "- my bullet" and "+200 %" in impact and "+100 %" not in impact
    assert cf.problems(path.read_text(encoding="utf-8")) == []


def test_the_gate_accepts_a_fragment_or_a_changelog_entry_and_fails_on_neither(tmp_path, monkeypatch):
    """The gate can fail: a table that no fragment and no CHANGELOG.md holds is not recorded."""
    monkeypatch.setattr(cf, "CHANGES", tmp_path / "changes")
    monkeypatch.setattr(di, "fragment_holds", lambda t: cf.fragment_holds(t, tmp_path / "changes"))
    log = tmp_path / "CHANGELOG.md"
    log.write_text("# Changelog\n", encoding="utf-8")
    monkeypatch.setattr(di, "CHANGELOG", log)
    assert di.recorded(TABLE) is False
    cf.write_impact("ticket/1-x", IMPACT, tmp_path / "changes")
    assert di.recorded(TABLE) is True                      # a fragment
    for p in (tmp_path / "changes").iterdir():
        p.unlink()
    assert di.recorded(TABLE) is False
    log.write_text("# Changelog\n\n" + TABLE + "\n", encoding="utf-8")
    assert di.recorded(TABLE) is True                      # the old form, during the move


def git(repo, *args):
    return subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True, check=True).stdout


def make_repo(tmp_path):
    repo = tmp_path / "repo"
    repo.mkdir()
    git(repo, "init", "-q", "-b", "main")
    git(repo, "config", "user.email", "t@example.invalid")
    git(repo, "config", "user.name", "t")
    (repo / "CHANGELOG.md").write_text(LOG, encoding="utf-8")
    (repo / "changes").mkdir()
    (repo / "changes" / "README.md").write_text("x\n", encoding="utf-8")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "base")
    return repo


def test_two_branches_that_each_add_a_fragment_merge_without_a_conflict(tmp_path):
    repo = make_repo(tmp_path)
    for name in ("a", "b"):
        git(repo, "checkout", "-q", "-b", name, "main")
        cf.write_impact(f"ticket/{name}", IMPACT.replace("ticket/1-x", f"ticket/{name}"), repo / "changes")
        git(repo, "add", "-A")
        git(repo, "commit", "-q", "-m", name)
    git(repo, "checkout", "-q", "main")
    git(repo, "merge", "-q", "--no-edit", "a")
    git(repo, "merge", "-q", "--no-edit", "b")                 # a conflict would raise here
    assert sorted(p.name for p in cf.fragments(repo / "changes")) == ["ticket-a.md", "ticket-b.md"]


def test_two_branches_that_each_edit_changelog_conflict(tmp_path):
    """The reason for fragments: the old form conflicts."""
    repo = make_repo(tmp_path)
    for name in ("a", "b"):
        git(repo, "checkout", "-q", "-b", name, "main")
        (repo / "CHANGELOG.md").write_text(LOG.replace("## Unreleased\n\n", f"## Unreleased\n\n- {name}\n\n"),
                                           encoding="utf-8")
        git(repo, "commit", "-q", "-am", name)
    git(repo, "checkout", "-q", "main")
    git(repo, "merge", "-q", "--no-edit", "a")
    r = subprocess.run(["git", "-C", str(repo), "merge", "-q", "--no-edit", "b"], capture_output=True, text=True)
    assert r.returncode != 0


def test_a_merge_group_needs_a_well_formed_fragment_not_one_matching_table(tmp_path, monkeypatch):
    repo = make_repo(tmp_path)
    monkeypatch.setattr(di, "ROOT", repo)
    monkeypatch.setattr(di, "git", lambda *a, **k: git(repo, *a))
    base = git(repo, "rev-parse", "HEAD").strip()
    git(repo, "checkout", "-q", "-b", "group")
    assert di.merge_group_problems(base) != []                 # no fragment at all: fails
    (repo / "changes" / "bad.md").write_text("loose text\n", encoding="utf-8")
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "bad")
    assert any("bad.md" in p for p in di.merge_group_problems(base))
    (repo / "changes" / "bad.md").unlink()
    cf.write_impact("ticket/g", IMPACT, repo / "changes")      # a table no PR's own run produced
    git(repo, "add", "-A")
    git(repo, "commit", "-q", "-m", "good")
    assert di.merge_group_problems(base) == []


def test_the_changes_folder_holds_a_readme_the_collector_leaves_alone():
    assert (ROOT / "changes" / "README.md").is_file()
    assert all(p.name != "README.md" for p in cf.fragments())
