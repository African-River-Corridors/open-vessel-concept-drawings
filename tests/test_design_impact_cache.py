"""The design-impact check keeps the answers of a commit for a case set (issue #561).

Label: property
"""
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("di_cache", ROOT / "tools" / "design_impact.py")
di = importlib.util.module_from_spec(spec)
spec.loader.exec_module(di)

CASES = {"a": {"x": 1}, "b": {"x": 2}}
ANSWERS = {"a": {"results": {"v": 1.5}}, "b": {"error": "no results"}}


def test_no_cache_folder_means_no_cache(monkeypatch):
    monkeypatch.delenv("DESIGN_IMPACT_CACHE", raising=False)
    assert di.load_cached("abc", di.cases_digest(CASES)) is None


def test_answers_come_back_only_for_the_same_commit_and_case_set(monkeypatch, tmp_path):
    monkeypatch.setenv("DESIGN_IMPACT_CACHE", str(tmp_path))
    monkeypatch.setattr(di, "git", lambda *a, **k: "")          # a clean tree
    digest = di.cases_digest(CASES)
    di.save_cached("abc", digest, ANSWERS)
    assert di.load_cached("abc", digest) == ANSWERS
    assert di.load_cached("abd", digest) is None
    assert di.load_cached("abc", di.cases_digest({**CASES, "c": {"x": 3}})) is None


def test_a_file_that_names_another_commit_is_not_used(monkeypatch, tmp_path):
    monkeypatch.setenv("DESIGN_IMPACT_CACHE", str(tmp_path))
    digest = di.cases_digest(CASES)
    (tmp_path / f"abc-{digest}.json").write_text(
        json.dumps({"commit": "zzz", "digest": digest, "answers": ANSWERS}), encoding="utf-8")
    assert di.load_cached("abc", digest) is None
    (tmp_path / f"abc-{digest}.json").write_text("not json", encoding="utf-8")
    assert di.load_cached("abc", digest) is None


def test_a_tree_with_edits_writes_nothing(monkeypatch, tmp_path):
    monkeypatch.setenv("DESIGN_IMPACT_CACHE", str(tmp_path))
    monkeypatch.setattr(di, "git", lambda *a, **k: " M crates/x.rs")
    di.save_cached("abc", di.cases_digest(CASES), ANSWERS)
    assert list(tmp_path.iterdir()) == []
