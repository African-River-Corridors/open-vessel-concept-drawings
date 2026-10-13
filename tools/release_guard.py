"""Release guard: fail if the tree, or the history, holds an embargoed term or a machine-local path.

    python3 tools/release_guard.py              # every file `git ls-files` lists
    python3 tools/release_guard.py --history    # every line added in `git log -p --all`,
                                                # plus commit authors and messages

The terms live in `tools/embargo.py` as SHA-256 hashes, so this public repo does not publish
them; matching is whole-word, case-insensitive (see that module). Prints one `where:line:term-id`
per hit and exits 1 if any hit is not allowlisted. In history mode `where` is `<commit>:<path>`,
with `<commit>:(commit)` for the author, committer and message. Standard library only.

Allowlist: `.release-guard-allow` at the repo root, one entry per line:

    <path-glob>  <term>  # reason

`<path-glob>` is matched with fnmatch against the repo-relative path. `<term>` is a term id as
the guard prints it, a path-rule name (`/Users/`), or `*` for every term. Blank lines and lines
starting with `#` are ignored. The guard does not scan itself or the allowlist file.
"""
from __future__ import annotations

import fnmatch
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import embargo  # noqa: E402

ALLOW_FILE = ".release-guard-allow"
SELF = ("tools/release_guard.py", "tools/embargo.py")
SKIP = (*SELF, ALLOW_FILE)
COMMIT = "(commit)"


def load_allow(root: Path) -> list[tuple[str, str]]:
    path = root / ALLOW_FILE
    if not path.exists():
        return []
    entries = []
    for n, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        parts = line.split()
        if len(parts) < 2:
            sys.exit(f"{ALLOW_FILE}:{n}: need '<path-glob>  <term>  # reason'")
        glob, term = parts[0], " ".join(parts[1:])
        tid = embargo.known_id(term)
        if tid is None:
            sys.exit(f"{ALLOW_FILE}:{n}: unknown term {term!r}")
        entries.append((glob, tid))
    return entries


def allowed(path: str, tid: str, allow: list[tuple[str, str]]) -> bool:
    return any(fnmatch.fnmatch(path, g) and t in ("*", tid) for g, t in allow)


def _scan_text(where: str, path: str, lines, allow, hashes, hits, ok) -> None:
    for n, line in lines:
        for tid, _ in embargo.find(line, hashes):
            (ok if allowed(path, tid, allow) else hits).append(f"{where}:{n}:{tid}")


def scan(root: Path, hashes: frozenset[str] | None = None) -> tuple[list[str], list[str]]:
    """Tree mode: (blocking hits, allowlisted hits), each as 'file:line:term-id'."""
    files = subprocess.run(["git", "ls-files", "-z"], cwd=root, check=True,
                           capture_output=True).stdout.decode().split("\0")
    allow = load_allow(root)
    hits, ok = [], []
    for rel in filter(None, files):
        if rel in SKIP:
            continue
        p = root / rel
        if not p.is_file():
            continue
        data = p.read_bytes()
        if b"\0" in data[:8192]:
            continue                                   # binary
        text = data.decode("utf-8", errors="replace")
        _scan_text(rel, rel, enumerate(text.splitlines(), 1), allow, hashes, hits, ok)
    return hits, ok


def _git(root: Path, *args: str) -> str:
    return subprocess.run(["git", "-c", "core.quotepath=off", *args], cwd=root, check=True,
                          capture_output=True).stdout.decode("utf-8", errors="replace")


def scan_history(root: Path, hashes: frozenset[str] | None = None) -> tuple[list[str], list[str]]:
    """History mode: every added line in every commit reachable from any ref, plus each commit's
    author, committer and message. Hits as '<commit12>:<path>:<line>:<term-id>'."""
    allow = load_allow(root)
    hits, ok = [], []

    meta = _git(root, "log", "--all", "--format=%H%x00%an <%ae>%n%cn <%ce>%n%B%x01")
    for rec in filter(str.strip, meta.split("\x01")):
        sha, _, body = rec.strip("\n").partition("\x00")
        _scan_text(f"{sha[:12]}:{COMMIT}", COMMIT, enumerate(body.splitlines(), 1),
                   allow, hashes, hits, ok)

    patch = _git(root, "log", "--all", "-p", "--no-color", "--no-ext-diff", "-U0",
                 "--format=commit %H")
    sha, path, lines, n, header = "", None, [], 0, False

    def flush():
        if path and path not in SKIP:
            _scan_text(f"{sha[:12]}:{path}", path, lines, allow, hashes, hits, ok)

    for raw in patch.splitlines():
        if raw.startswith("commit ") and len(raw) == 47:
            flush()
            sha, path, lines, header = raw[7:], None, [], False
        elif raw.startswith("diff --git "):
            flush()
            path, lines, header = None, [], True
        elif header and raw.startswith("+++ "):
            target = raw[4:]
            path = None if target == "/dev/null" else target[2:] if target.startswith("b/") else target
        elif raw.startswith("@@"):
            header = False
            try:                                        # "@@ -a,b +c,d @@": added lines start at c
                n = int(raw.split("+", 1)[1].split()[0].split(",")[0])
            except (IndexError, ValueError):
                n = 0
        elif not header and raw.startswith("+") and path is not None:
            lines.append((n, raw[1:]))
            n += 1
    flush()
    return hits, ok


def main(argv: list[str]) -> int:
    root = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], check=True,
                               capture_output=True, text=True).stdout.strip())
    history = "--history" in argv
    hits, ok = (scan_history if history else scan)(root)
    for h in hits:
        print(h)
    mode = "history" if history else "tree"
    print(f"release guard ({mode}): {len(hits)} hit(s), {len(ok)} allowlisted", file=sys.stderr)
    return 1 if hits else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
