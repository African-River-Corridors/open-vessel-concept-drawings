"""Write the derived `assess` inputs for the gas_stability port (issue #18).

    python3 parity/gas_basis_fixtures.py crates/ovcd-format/tests/fixtures/gas_stability_results.json

`gas_stability.assess` takes a converged gas arrangement. The parity cases describe it by `fit` (the
`gas_arrangement.fit` entry point) or `capacity` (`barge_capacity.compute`). Neither is ported yet.
This script runs the frozen Python on those descriptions and records the plain numbers `assess`
reads from the result. The Rust test feeds these numbers to the Rust `assess` and compares it with
the Reference Results. It does not write a Reference Result.

Delete this script and the fixture when `assess` runs through `eval` (after `gas_arrangement` and
`barge_capacity` are ported).
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

PARITY = Path(__file__).resolve().parent
sys.path.insert(0, str(PARITY))
import driver  # noqa: E402


def derived(args: dict) -> dict:
    args = dict(args)
    if "fit" in args:
        f = driver._module("core.gas_arrangement").fit(**args.pop("fit"))
        result, payload = f.engine_result, args.pop("payload_t", f.payload_t)
    else:
        cap = driver._module("core.barge_capacity")
        result = cap.compute(driver.build(cap.BargeCapacityInputs, args.pop("capacity")))
        payload = args.pop("payload_t")
    basis = result.gas_basis
    out = {
        "cargo_density_t_per_m3": result.cargo_density_t_per_m3,
        "lightship_t": result.lightship_t,
        "has_aft_arrangement": getattr(result, "aft_arrangement", None) is not None,
        "payload_t": payload,
        "gas_basis": None,
    }
    if basis is not None:
        bi, t, w = basis["inputs"], basis["tanks"], basis["weights"]
        out["gas_basis"] = {
            "loa": bi.loa, "beam": bi.beam, "depth": bi.depth,
            "double_bottom_height": bi.double_bottom_height,
            "tank_support_height": bi.tank_support_height,
            "outer_diameter": t.outer_diameter, "inner_diameter": t.inner_diameter,
            "total_tank_volume": t.total_tank_volume, "n_tanks_total": t.n_tanks_total,
            "cylinder_length": t.cylinder_length,
            "steel_top_down": w.steel_top_down, "tank_total": w.tank_total,
            "insulation": w.insulation, "piping_machinery": w.piping_machinery,
            "outfitting": w.outfitting,
        }
    return out


def main(out_path: str) -> None:
    cases = json.loads((PARITY / "cases" / "gas_stability.json").read_text())["cases"]
    rows = []
    for case in cases:
        inp = case["input"]
        if inp["fn"] != "assess":
            continue
        rows.append({"id": case["id"], "result": derived(inp["args"])})
    Path(out_path).write_text(json.dumps({"rows": rows}, indent=1, allow_nan=False) + "\n")


if __name__ == "__main__":
    main(sys.argv[1])
