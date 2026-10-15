"""Write tests/data/api_ce96e66.json: every API scenario of tests/api_scenarios.py, run on the
frozen pure-Python package at commit ce96e66.

The frozen package is not in the tree (it is in the history). To regenerate the file:

    git worktree add /tmp/oracle ce96e66
    python tests/data/make_api_ce96e66.py /tmp/oracle/parity/frozen_python tests/data/api_ce96e66.json

The directory given holds the package `vessel_designer_frozen`. Python 3.12 with numpy. Nobody edits
the JSON by hand: a value is printed by the original code or it is not there.

A mode whose answer equals the `float` mode answer is not repeated (`90 == 90.0` in Python), so
the file holds each result once unless the frozen package gave a different one.
"""
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent


def main(frozen_root: str, out: str) -> None:
    sys.path.insert(0, frozen_root)
    sys.path.insert(0, str(HERE.parent))
    import api_scenarios as S

    scenarios = {}
    for name in S.SCENARIOS:
        results = {mode: S.run("vessel_designer_frozen", mode, name) for mode in S.MODES}
        scenarios[name] = {"float": results["float"]}
        scenarios[name].update({m: r for m, r in results.items() if m != "float" and r != results["float"]})
    lines = ['{', ' "_about": "Printed by the frozen Python at commit ce96e66 with tests/data/make_api_ce96e66.py. Do not edit by hand.",',
             ' "scenarios": {']
    items = [f'  {json.dumps(n)}: {json.dumps(r, separators=(",", ":"))}' for n, r in scenarios.items()]
    lines.append(",\n".join(items))
    lines += [" }", "}"]
    Path(out).write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"{len(scenarios)} scenarios -> {out}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
