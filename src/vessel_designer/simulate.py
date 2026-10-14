"""One-call barge simulation for the web simulator (runs in the browser via Pyodide).

    simulate(params: dict) -> dict      # JSON in, JSON out

Adds the weight limit the core leaves out: the core sizes cargo by HOLD VOLUME; a real barge
also stops at its draught limit. Cargo = the smaller of the two, and the result says which
governed. Pushed barges only for now; the pusher is a separate vessel and is not included.
The calculation runs in the Rust core (`vessel_designer._native`); this module only passes the
parameters through. Pure calculation, no I/O.
"""
from __future__ import annotations

from ._bridge import call as _call

RHO = 1.0  # fresh water, t/m3
# Tank filling limit by ruleset (defect 7): IGC 15.3 default 98 %, ADN 95 %.
FILL_BY_RULESET = {"IGC_2G": 0.98, "IGC_2G_EXACT": 0.98, "ADN_G": 0.95}

VESSELS = {
    # id: (label, hold_type for the core, allowed cargo phases)
    "box_barge": ("Box-hold dry bulk barge", "box", {"dry_bulk"}),
    "hopper_barge": ("Sloped-hopper dry bulk barge", "sloped_hopper", {"dry_bulk"}),
    "gas_barge": ("Gas barge, IMO Type C tanks", None, {"liquid_gas"}),
}


def cargo_options() -> list[dict]:
    """Every cargo the simulator offers: id, name, phase and density."""
    return _call("simulate", "cargo_options", {})


def simulate(p: dict) -> dict:
    """Run the one-call simulation on the parameters ``p`` and return the result as a dict."""
    return _call("simulate", "simulate", {"params": p})
