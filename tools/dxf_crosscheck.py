"""The DXF files of the drawing crate, read by readers that are not the crate (spec 0003, issue #99).

For every conformance document that has a result, and every view of every fixture Design, `ovd draw build`
(or `ovd draw --out`) writes a DXF. Then:

1. the standard's own test reader (`tests/dxf_reader.py` of the standard's repository, which works on the
   group codes and does not use ezdxf) rebuilds the normalised content from the DXF. It must equal the
   normalised content of the Python implementation, and the viewports must hold the stated scales;
2. ezdxf reads the file and its audit finds no error.

    python3 tools/dxf_crosscheck.py --ovd target/debug/ovd --tdfa-src <checkout of the standard's repository>

Needs `ezdxf` and the Python implementation of the standard (`pip install` at the pinned commit; see the
`drawing-crosscheck` job in .github/workflows/ci.yml). Exit 1 on any difference.
"""
from __future__ import annotations

import argparse
import io
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import otd  # noqa: E402  the pinned crate, for its conformance suite

ROOT = Path(__file__).resolve().parent.parent
SUITE = otd.standard() / "conformance"
FIXTURES = ROOT / "crates" / "ovd-format" / "tests" / "fixtures"
MIN_FILES = 100


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--ovd", default=str(ROOT / "target" / "debug" / "ovd"))
    ap.add_argument("--tdfa-src", required=True, help="a checkout of the standard's repository (for tests/dxf_reader.py)")
    args = ap.parse_args()
    sys.path.insert(0, args.tdfa_src)
    import ezdxf  # noqa: PLC0415
    from tests.dxf_reader import read_normalised  # noqa: PLC0415
    from technical_drawings_for_agents.dfs.conformance import _same  # noqa: PLC0415
    from technical_drawings_for_agents.dfs.pipeline import run_file, sort_key  # noqa: PLC0415

    jobs: list[tuple[str, Path, Path]] = []  # label, document, dxf
    problems: list[str] = []
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        suite = json.loads((SUITE / "suite.json").read_text())
        for case in suite["cases"]:
            if case["kind"] != "document":
                continue
            expected = json.loads((SUITE / case["dir"] / "expected.json").read_text())
            if expected["result"] != "ok":
                continue
            work = tmp / case["id"]
            shutil.copytree(SUITE / case["dir"], work)
            proc = subprocess.run([args.ovd, "draw", "build", str(work / "input.json"), "--out", str(work / "out")], capture_output=True, text=True)
            if proc.returncode != 0:
                problems.append(f"{case['id']}: ovd draw build failed: {proc.stderr.strip()[:300]}")
                continue
            jobs.append((case["id"], work / "input.json", work / "out" / "input.dxf"))
        for fixture in sorted(FIXTURES.glob("*.vdesign")):
            out = tmp / fixture.stem
            proc = subprocess.run([args.ovd, "draw", "--out", str(out), str(fixture)], capture_output=True, text=True)
            if proc.returncode != 0:
                if "no drawings of a pushed convoy yet" in proc.stderr:
                    continue
                problems.append(f"{fixture.name}: ovd draw failed: {proc.stderr.strip()[:300]}")
                continue
            for document in sorted(out.glob("*.drawing.json")):
                jobs.append((f"{fixture.name}/{document.name}", document, document.with_name(document.name[: -len(".drawing.json")] + ".dxf")))

        for label, document, dxf in jobs:
            text = dxf.read_text(encoding="utf-8")
            norm = run_file(document)["normalised"]
            back = read_normalised(text)
            want = sorted(norm["entities"], key=sort_key)
            got = sorted(back["entities"], key=sort_key)
            if len(got) != len(want) or not all(_same(g, w) for g, w in zip(got, want)):
                problems.append(f"{label}: the DXF read back gives other entities ({len(got)} found, {len(want)} expected)")
                continue
            for vp in norm["viewports"]:
                b = back["viewports"].get(vp["id"])
                frame = [vp["frame"][k] for k in ("x_mm", "y_mm", "width_mm", "height_mm")]
                if not b or b["n"] != vp["scale"] or [round(c, 6) for c in b["frame"]] != frame:
                    problems.append(f"{label}: viewport {vp['id']} does not hold its scale or frame")
            try:
                errors = ezdxf.read(io.StringIO(text)).audit().errors
            except Exception as e:  # noqa: BLE001
                problems.append(f"{label}: ezdxf cannot read the file: {e!r}")
                continue
            if errors:
                problems.append(f"{label}: ezdxf audit finds {len(errors)} error(s): {errors[0]}")
    for p in problems:
        print("FAIL", p)
    print(f"{len(jobs)} DXF files read back by the standard's reader and ezdxf; {len(problems)} problem(s)")
    if len(jobs) < MIN_FILES and not problems:
        print(f"FAIL: only {len(jobs)} files were checked, expected at least {MIN_FILES}")
        return 1
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
