#!/usr/bin/env python3
"""The old-name gate (spec 0010): the tool's name before the rename to OVD appears only as history.

    python3 tools/old_name_gate.py

Fails when a tracked text file holds the old short name, in any case, outside:
  - the history: CHANGELOG.md, changes/, docs/adr/ (earlier ADRs), spec 0010 itself;
  - this file;
  - the 0.2.4 file that proves an old file does not open (crates/ovd-format/tests/legacy/before_ovd_*);
  - a line marked `before-ovd` (the error that names the rename, and its test).
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OLD = "ov" + "cd"  # split, so this file needs no exemption for the pattern itself
HISTORY = re.compile(
    r"^(CHANGELOG\.md|changes/|docs/adr/|docs/specs/0010-rename-to-open-vessel-designer\.md"
    r"|tools/old_name_gate\.py|crates/ovd-format/tests/legacy/before_ovd_)"
)
MARK = "before-ovd"


SKIP_DIRS = {".git", "target", "node_modules", ".venv", "_site", "__pycache__"}


def tracked() -> list[str]:
    """The tracked files, or every file under the root when it is not a git checkout (a test copy)."""
    out = subprocess.run(["git", "-C", str(ROOT), "ls-files"], capture_output=True, text=True)
    if out.returncode == 0:
        return out.stdout.splitlines()
    return [
        p.relative_to(ROOT).as_posix()
        for p in ROOT.rglob("*")
        if p.is_file() and not SKIP_DIRS.intersection(p.relative_to(ROOT).parts)
    ]


def problems() -> list[str]:
    found = []
    for path in tracked():
        if HISTORY.match(path):
            continue
        if OLD in path.lower():
            found.append(f"path: {path}")
        data = (ROOT / path).read_bytes() if (ROOT / path).is_file() else b""
        if b"\0" in data[:8192]:
            continue  # binary
        for n, line in enumerate(data.decode("utf-8", "replace").splitlines(), 1):
            if OLD in line.lower() and MARK not in line:
                found.append(f"{path}:{n}:{line.strip()[:120]}")
    return found


def main() -> int:
    found = problems()
    for f in found:
        print(f"old_name_gate: FAIL: {f}", file=sys.stderr)
    if found:
        print(f"old_name_gate: {len(found)} use(s) of the old name outside the history (spec 0010)", file=sys.stderr)
        return 1
    print("old_name_gate: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
