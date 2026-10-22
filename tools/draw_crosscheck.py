"""Cross-implementation check of the drawing crate (spec 0003, issue #100).

For every fixture Design, `ovcd draw --document` prints the drawing documents of each Option. Each
document is run through the Rust implementation of the Drawing Format Standard (`ovcd draw run`) and
through the Python implementation (`technical_drawings_for_agents run`). Both must read the document,
and their normalised content must be equal (clause 16.9: numbers are compared as numbers).

    python3 tools/draw_crosscheck.py [--ovcd target/debug/ovcd] [--tool technical_drawings_for_agents]

The Python tool is installed at the commit that the pinned `otd` crate's standard/PIN.json records
(`python3 tools/otd.py pin`; see the `drawing-crosscheck` job in .github/workflows/ci.yml). Exit 1 on any difference, and on a run that
compared too few documents to mean anything. Standard library only.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "crates" / "ovcd-format" / "tests" / "fixtures"
MIN_DOCUMENTS = 15  # five computed Options with three views each, at least


def same(a, b) -> bool:
    """Clause 16.9: parsed values are equal; 2 and 2.0 are the same number."""
    if isinstance(a, bool) or isinstance(b, bool):
        return a is b
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return a == b
    if isinstance(a, dict) and isinstance(b, dict):
        return a.keys() == b.keys() and all(same(a[k], b[k]) for k in a)
    if isinstance(a, list) and isinstance(b, list):
        return len(a) == len(b) and all(same(x, y) for x, y in zip(a, b))
    return a == b


def first_difference(a, b, path="") -> str:
    if isinstance(a, dict) and isinstance(b, dict):
        for k in sorted(set(a) | set(b)):
            if k not in a or k not in b:
                return f"{path}/{k} is missing on one side"
            if not same(a[k], b[k]):
                return first_difference(a[k], b[k], f"{path}/{k}")
    if isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            return f"{path} has {len(a)} items, then {len(b)}"
        for i, (x, y) in enumerate(zip(a, b)):
            if not same(x, y):
                return first_difference(x, y, f"{path}[{i}]")
    return f"{path}: {json.dumps(a)[:200]} != {json.dumps(b)[:200]}"


def run_json(command: list[str]) -> dict:
    proc = subprocess.run(command, capture_output=True, text=True)
    if proc.returncode != 0 or not proc.stdout.strip():
        raise SystemExit(f"FAIL: {' '.join(command)} gave exit {proc.returncode}: {proc.stderr.strip()[:500]}")
    return json.loads(proc.stdout)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ovcd", default=str(ROOT / "target" / "debug" / "ovcd"))
    ap.add_argument("--tool", default="technical_drawings_for_agents")
    args = ap.parse_args()

    compared = 0
    problems: list[str] = []
    with tempfile.TemporaryDirectory() as tmp:
        for fixture in sorted(FIXTURES.glob("*.vdesign")):
            proc = subprocess.run([args.ovcd, "draw", "--document", str(fixture)], capture_output=True, text=True)
            if proc.returncode != 0:
                # a convoy has no drawings yet (issue #358): that is the expected message, nothing else is
                if "no drawings of a pushed convoy yet" in proc.stderr and json.loads(proc.stdout or "{}") == {}:
                    print(f"{fixture.name}: no views (convoy), as expected")
                    continue
                problems.append(f"{fixture.name}: ovcd draw --document failed: {proc.stderr.strip()[:300]}")
                continue
            for option, views in json.loads(proc.stdout).items():
                for view, document in views.items():
                    path = Path(tmp) / f"{fixture.stem}-{len(list(Path(tmp).iterdir()))}-{view}.json"
                    path.write_text(json.dumps(document))
                    rust = run_json([args.ovcd, "draw", "run", str(path)])
                    python = run_json([args.tool, "run", str(path)])
                    label = f"{fixture.name} / {option} / {view}"
                    if rust["result"] != "ok" or python["result"] != "ok":
                        problems.append(f"{label}: rust {rust['result']} {rust.get('errors')}, python {python['result']} {python.get('errors')}")
                    elif not same(rust["normalised"], python["normalised"]):
                        problems.append(f"{label}: normalised content differs at {first_difference(rust['normalised'], python['normalised'])}")
                    else:
                        compared += 1
    for p in problems:
        print("FAIL", p)
    print(f"{compared} documents have equal normalised content in both implementations")
    if compared < MIN_DOCUMENTS and not problems:
        print(f"FAIL: only {compared} documents were compared, expected at least {MIN_DOCUMENTS}")
        return 1
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
