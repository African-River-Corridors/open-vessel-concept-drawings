#!/usr/bin/env python3
"""Run `ovcd run --json` on the pinned fixture designs and fail on any results-hash mismatch.

    python3 tools/check_fixture_hashes.py <path to ovcd> [<fixtures directory>]

The fixtures in crates/ovcd-format/tests/fixtures/*.vdesign store the results hash of each Option.
The hash depends on bit-identical maths on every system (spec 0001, "platform-identical maths").
`cargo test` checks it on Linux. The release gate runs this on Windows and on Intel and Apple-silicon
macOS, with the release binary, so a platform whose maths differs cannot be released.

It fails when an Option's `hash_status` is anything but "matches", when a fixture that should compute
does not, when `ovcd` exits non-zero (except for a convoy), or when fewer than MIN_MATCHED Options matched (so an empty
fixture directory cannot pass). A pushed convoy has no numbers yet and is the one allowed exception.
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "crates" / "ovcd-format" / "tests" / "fixtures"
NO_RESULTS_YET = {"pushed_convoy.vdesign"}      # convoy results are not available yet
MIN_MATCHED = 6                                 # five computable fixtures, one of them with two Options


def find_binary(path: str) -> Path:
    p = Path(path)
    for candidate in (p, p.with_name(p.name + ".exe")):
        if candidate.is_file():
            return candidate
    raise SystemExit(f"check_fixture_hashes: no ovcd binary at {path}")


def check(binary: Path, fixtures: Path) -> list[str]:
    """Problems found, one line each. Prints one line per Option."""
    files = sorted(fixtures.glob("*.vdesign"))
    problems: list[str] = []
    if not files:
        return [f"no .vdesign fixtures in {fixtures}"]
    matched = 0
    for f in files:
        r = subprocess.run([str(binary), "run", "--json", str(f)], capture_output=True,
                           encoding="utf-8", errors="replace")
        # A pushed convoy has no results, so `ovcd run` exits 1 for it (not every Option computed).
        if r.returncode != 0 and f.name not in NO_RESULTS_YET:
            problems.append(f"{f.name}: ovcd exited {r.returncode}: {r.stderr.strip()[-200:]}")
            continue
        try:
            options = json.loads(r.stdout)["options"]
        except (ValueError, KeyError):
            problems.append(f"{f.name}: ovcd printed no options")
            continue
        for o in options:
            name, status = o.get("name"), o.get("hash_status")
            if o.get("results") is None:
                if f.name in NO_RESULTS_YET:
                    print(f"  {f.name} / {name}: no results yet (allowed)")
                else:
                    problems.append(f"{f.name} / {name}: no results")
                continue
            digest = (o.get("results_hash") or "")[:12]
            print(f"  {f.name} / {name}: {status} {digest}")
            if status == "matches":
                matched += 1
            else:
                problems.append(f"{f.name} / {name}: hash status is {status!r}, not 'matches'")
    if matched < MIN_MATCHED:
        problems.append(f"only {matched} Options matched; at least {MIN_MATCHED} are expected")
    return problems


def main(argv: list[str]) -> int:
    if not 2 <= len(argv) <= 3:
        print(__doc__, file=sys.stderr)
        return 2
    binary = find_binary(argv[1])
    fixtures = Path(argv[2]) if len(argv) == 3 else FIXTURES
    print(f"check_fixture_hashes: {binary}")
    problems = check(binary, fixtures)
    for p in problems:
        print(f"check_fixture_hashes: FAIL: {p}", file=sys.stderr)
    if not problems:
        print("check_fixture_hashes: every fixture hash matches")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
