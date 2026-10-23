#!/usr/bin/env python3
"""Run `ovcd run --json` on the pinned fixture designs and fail on any results-hash mismatch.

    python3 tools/check_fixture_hashes.py <path to ovcd> [<fixtures directory>]
    python3 tools/check_fixture_hashes.py --wasmtime <path to wasmtime> <path to ovcd.wasm> [<fixtures directory>]

The fixtures in crates/ovcd-format/tests/fixtures/*.vdesign store the results hash of each Option. With no
fixtures directory given, the website examples (website/assets/examples/*.vdesign) are checked too: a
results change that does not re-stamp them must fail here, not on the website.
The hash depends on bit-identical maths on every system (spec 0001, "platform-identical maths").
`cargo test` checks it on Linux. The release gate runs this on Windows and on Intel and Apple-silicon
macOS, with the release binary, and on WebAssembly: the same CLI built for `wasm32-wasip1` and run
under `wasmtime` (issue #168). So a platform whose maths differs cannot be released.

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
EXAMPLES = ROOT / "website" / "assets" / "examples"
NO_RESULTS_YET = {"pushed_convoy.vdesign"}      # convoy results are not available yet
MIN_MATCHED = 6                                 # five computable fixtures, one of them with two Options


def find_binary(path: str) -> Path:
    p = Path(path)
    for candidate in (p, p.with_name(p.name + ".exe")):
        if candidate.is_file():
            return candidate
    raise SystemExit(f"check_fixture_hashes: no ovcd binary at {path}")


def command_for(binary: Path, fixtures: Path, wasmtime: Path | None, file: Path) -> list[str]:
    """The command that runs one fixture: the binary itself, or the wasm module under wasmtime."""
    if wasmtime is None:
        return [str(binary), "run", "--json", str(file)]
    # The module sees only the folder of the file (same path inside as outside).
    return [str(wasmtime), "run", "--dir", str(file.parent), str(binary), "run", "--json", str(file)]


def check(binary: Path, fixtures: Path, wasmtime: Path | None = None, also: tuple[Path, ...] = ()) -> list[str]:
    """Problems found, one line each. Prints one line per Option. `also`: more folders of designs."""
    files = sorted(fixtures.glob("*.vdesign"))
    problems: list[str] = []
    if not files:
        return [f"no .vdesign fixtures in {fixtures}"]
    for folder in also:
        more = sorted(folder.glob("*.vdesign"))
        if not more:
            return [f"no .vdesign designs in {folder}"]
        files += more
    matched = 0
    for f in files:
        r = subprocess.run(command_for(binary, fixtures, wasmtime, f), capture_output=True,
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
    args = argv[1:]
    wasmtime: Path | None = None
    if args[:1] == ["--wasmtime"]:
        if len(args) < 2:
            print(__doc__, file=sys.stderr)
            return 2
        wasmtime = Path(args[1])
        if not wasmtime.is_file():
            raise SystemExit(f"check_fixture_hashes: no wasmtime at {wasmtime}")
        args = args[2:]
    if not 1 <= len(args) <= 2:
        print(__doc__, file=sys.stderr)
        return 2
    binary = find_binary(args[0])
    fixtures = Path(args[1]) if len(args) == 2 else FIXTURES
    also = () if len(args) == 2 else (EXAMPLES,)
    print(f"check_fixture_hashes: {binary}" + (f" under {wasmtime}" if wasmtime else ""))
    problems = check(binary, fixtures, wasmtime, also)
    for p in problems:
        print(f"check_fixture_hashes: FAIL: {p}", file=sys.stderr)
    if not problems:
        print("check_fixture_hashes: every fixture hash matches")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
