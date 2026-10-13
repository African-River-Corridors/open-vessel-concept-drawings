"""The release guard can fail: it finds a planted term in the tree and in history, honours the
allowlist, and matches whole words only.

The real embargo list is hashed, so these tests plant made-up terms and pass their hashes in.
They exercise the same code path the shipped list uses.
"""
import importlib.util
import os
import subprocess
from pathlib import Path

TOOLS = Path(__file__).resolve().parents[1] / "tools"


def _load(name):
    spec = importlib.util.spec_from_file_location(name, TOOLS / f"{name}.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


rg = _load("release_guard")
emb = rg.embargo

WORD, PHRASE, SHORT = "zorblax", "quux harbour", "qzr"
H = frozenset(emb.term_hash(t) for t in (WORD, PHRASE, SHORT))
ID = {t: emb.term_id(t) for t in (WORD, PHRASE, SHORT)}

ENV = {**os.environ, "GIT_AUTHOR_NAME": "t", "GIT_AUTHOR_EMAIL": "t@example.com",
       "GIT_COMMITTER_NAME": "t", "GIT_COMMITTER_EMAIL": "t@example.com"}


def _git(root, *args):
    subprocess.run(["git", "-C", str(root), *args], check=True, env=ENV, capture_output=True)


def _repo(tmp_path, files, allow=""):
    subprocess.run(["git", "init", "-q", str(tmp_path)], check=True)
    for name, text in files.items():
        (tmp_path / name).parent.mkdir(parents=True, exist_ok=True)
        (tmp_path / name).write_text(text)
    if allow:
        (tmp_path / rg.ALLOW_FILE).write_text(allow)
    _git(tmp_path, "add", ".")
    return tmp_path


def _commit(root, msg="c"):
    _git(root, "commit", "-q", "-m", msg)


def test_hashed_list_does_not_hold_plain_text():
    assert emb.TERM_HASHES and all(len(h) == 64 and int(h, 16) >= 0 for h in emb.TERM_HASHES)


def test_tree_fails_on_planted_term(tmp_path):
    root = _repo(tmp_path, {"a.py": "x = 1\n# The ZORBLAX here\npath = '/Users/me/x'\n"})
    hits, _ = rg.scan(root, H)
    assert hits == [f"a.py:2:{ID[WORD]}", "a.py:3:/Users/"]


def test_tree_finds_a_multi_word_term_across_punctuation(tmp_path):
    root = _repo(tmp_path, {"a.md": "ok\nsee Quux-Harbour.\n"})
    assert rg.scan(root, H)[0] == [f"a.md:2:{ID[PHRASE]}"]


def test_tree_finds_a_phrase_wrapped_over_two_lines(tmp_path):
    root = _repo(tmp_path, {"a.md": "ok\nthe quux\nharbour project\n- quux\n  harbour\n"})
    assert rg.scan(root, H)[0] == [f"a.md:2:{ID[PHRASE]}", f"a.md:4:{ID[PHRASE]}"]


def test_a_wrapped_phrase_in_a_commit_message_is_found(tmp_path):
    root = _repo(tmp_path, {"a.md": "clean\n"})
    _commit(root, "fix for the quux\nharbour client")
    hits, _ = rg.scan_history(root, H)
    assert len(hits) == 1 and hits[0].endswith(f":(commit):3:{ID[PHRASE]}")


def test_a_wrapped_phrase_in_added_history_lines_is_found(tmp_path):
    root = _repo(tmp_path, {"a.md": "clean\nquux\nharbour\n"})
    _commit(root)
    (root / "a.md").write_text("clean\n")
    _git(root, "add", ".")
    _commit(root)
    hits, _ = rg.scan_history(root, H)
    assert len(hits) == 1 and hits[0].endswith(f":a.md:2:{ID[PHRASE]}")


def test_a_phrase_split_over_two_commits_is_found_in_history(tmp_path):
    root = _repo(tmp_path, {"a.md": "quux\nfiller\n"})
    for text in ("quux\nharbour\n", "x\n"):             # line 2 is edited next to an unchanged line
        (root / "a.md").write_text(text)
        _git(root, "add", ".")
        _commit(root)
    hits, _ = rg.scan_history(root, H)
    assert hits and all(h.endswith(f":a.md:1:{ID[PHRASE]}") for h in hits)


def test_a_local_path_in_the_embargo_module_is_a_hit_in_tree_and_history(tmp_path):
    root = _repo(tmp_path, {"tools/embargo.py": "# from /" + "Users/someone/terms.txt\n"})
    assert len(rg.scan(root, H)[0]) == 1
    _commit(root)
    assert len(rg.scan_history(root, H)[0]) == 1


def test_embargo_module_is_scanned_except_its_hash_literals(tmp_path):
    h = emb.term_hash(WORD)
    src = f'TERM_HASHES = frozenset({{\n    "{h}",\n}})\n# {WORD} hides here\n'
    root = _repo(tmp_path, {"tools/embargo.py": src, "tools/release_guard.py": f"# {WORD}\n"})
    hits, _ = rg.scan(root, H)
    assert hits == [f"tools/embargo.py:4:{ID[WORD]}"]       # the hash line is not a hit
    _commit(root)
    hits, _ = rg.scan_history(root, H)
    assert len(hits) == 1 and hits[0].endswith(f":tools/embargo.py:4:{ID[WORD]}")


def test_embargo_module_hash_literal_is_not_a_hit_when_the_term_is_listed(tmp_path):
    h = emb.term_hash(WORD)
    root = _repo(tmp_path, {"tools/embargo.py": f'X = (\n    "{h}",\n)\n'})
    assert rg.scan(root, H)[0] == []


def test_short_term_is_a_whole_word(tmp_path):
    root = _repo(tmp_path, {"a.md": "aqzrb qzrx\nthe QZR is 12%\nsnake_qzr_case\n"})
    hits, _ = rg.scan(root, H)
    assert hits == [f"a.md:2:{ID[SHORT]}", f"a.md:3:{ID[SHORT]}"]


def test_allowlist_by_glob_and_term_id(tmp_path, monkeypatch):
    monkeypatch.setattr(emb, "TERM_HASHES", H)
    root = _repo(tmp_path, {"docs/a.md": "zorblax\n", "b.md": "zorblax\n"},
                 allow=f"docs/*  {ID[WORD]}  # reason\n")
    hits, ok = rg.scan(root, H)
    assert hits == [f"b.md:1:{ID[WORD]}"] and ok == [f"docs/a.md:1:{ID[WORD]}"]


def test_untracked_files_are_not_scanned(tmp_path):
    root = _repo(tmp_path, {"a.md": "clean\n"})
    (root / "new.md").write_text("zorblax\n")
    assert rg.scan(root, H)[0] == []


def test_history_fails_on_a_term_removed_from_the_tree(tmp_path):
    root = _repo(tmp_path, {"a.md": "clean\nzorblax was here\n"})
    _commit(root)
    (root / "a.md").write_text("clean\n")
    _git(root, "add", ".")
    _commit(root)
    assert rg.scan(root, H)[0] == []                       # the tree is clean ...
    hits, _ = rg.scan_history(root, H)                     # ... the history is not
    assert len(hits) == 1 and hits[0].endswith(f":a.md:2:{ID[WORD]}")


def test_a_commit_qualified_allow_entry_excuses_history_but_not_the_tree(tmp_path, monkeypatch):
    monkeypatch.setattr(emb, "TERM_HASHES", H)
    root = _repo(tmp_path, {"a.md": "clean\nzorblax\n"})
    _commit(root)
    sha = subprocess.run(["git", "rev-parse", "HEAD"], cwd=root, capture_output=True,
                         text=True).stdout.strip()[:12]
    (root / ".release-guard-allow").write_text(f"{sha}:a.md  {ID[WORD]}  # one commit only\n")
    assert rg.scan_history(root, H)[0] == []
    assert len(rg.scan(root, H)[0]) == 1


def test_history_scans_commit_messages_and_authors(tmp_path):
    root = _repo(tmp_path, {"a.md": "clean\n"})
    _commit(root, "fix for the quux harbour client")
    hits, _ = rg.scan_history(root, H)
    assert len(hits) == 1 and hits[0].endswith(f":(commit):3:{ID[PHRASE]}")


def test_history_honours_the_allowlist(tmp_path, monkeypatch):
    monkeypatch.setattr(emb, "TERM_HASHES", H)
    root = _repo(tmp_path, {"docs/a.md": "zorblax\n"}, allow=f"docs/*  {ID[WORD]}  # reason\n")
    _commit(root)
    hits, ok = rg.scan_history(root, H)
    assert hits == [] and len(ok) == 1


def test_shipped_list_is_clean_on_this_tree():
    root = Path(__file__).resolve().parents[1]
    if (root / ".git").exists():
        assert rg.scan(root)[0] == []
