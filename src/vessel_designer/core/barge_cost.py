"""Bottom-up ATB unit cost (cargo barge + electric pusher tug) — from the mass model.

Takes the engine's own mass build-up (BargeCalculator: hull plating areas -> steel
tonnes; ASME/IGC pressure-vessel shell -> tank tonnes) and applies fabrication rates,
so the cost is quantity-based rather than a lump allowance. Scope: steel, the cryogenic
(Type-C) tanks and cargo handling; EXCLUDES reliquefaction and nitrogen plant
(pressure-accumulation service, with shore support at the terminals).

RATES ARE DATA, NOT CODE. This module holds no prices. Supply your own as a
``BargeCostRates`` object, a mapping, or a path to a rates file (same keys as the example
file). With no rates given, ``compute`` uses ``barge_cost_rates.example.yaml``: round,
illustrative numbers for the worked example only — not market quotes.

The rates file is flat YAML (``key: value  # comment``). The core reads that subset with the
standard library so it stays free of third-party imports; nested YAML is rejected loudly.

Pure Python (Pyodide-safe).
"""
from __future__ import annotations

from dataclasses import dataclass, field, fields
from importlib import resources
from os import PathLike
from pathlib import Path
from typing import Dict, List, Mapping, Optional, Union

from .checks import Check, all_ok
from .._bridge import call as _call

EXAMPLE_RATES_FILE = "barge_cost_rates.example.yaml"


@dataclass
class BargeCostInputs:
    # --- illustrative default masses for a ~90 m gas barge (BargeCalculator weights/tanks; t).
    # Replace with the mass model's output for your own hull. ---
    hull_steel_t: float = 253.2          # plating + internal structure
    wing_plate_t: float = 40.3
    weather_shield_t: float = 50.1
    tank_shell_t: float = 215.6          # Type-C cylinders + heads (25 mm, LT steel)
    tank_supports_t: float = 64.7        # saddles, anti-flotation, internals
    insulation_t: float = 25.9           # sprayed PU foam system
    piping_machinery_t: float = 32.9
    outfitting_t: float = 28.8

    # --- electric pusher tug (16 m pusher, modular swappable energy containers) ---
    tug_hull_steel_t: float = 115.0        # 16 m pusher, denser structure — ESTIMATE
    battery_kwh: float = 620.0             # sized on the longest inter-charge leg
    propulsion_kw: float = 230.0           # installed (binding P_bat x 1.2)


@dataclass(frozen=True)
class BargeCostRates:
    """Fabrication / supply rates. No defaults: every rate comes from data you supply."""
    hull_steel_rate: float               # per t of hull steel (barge and tug)
    tank_shell_rate: float               # per t of Type-C tank shell
    tank_supports_rate: float            # per t
    insulation_rate: float               # per t equivalent
    piping_rate: float                   # per t of piping + machinery
    outfitting_rate: float               # per t
    cargo_system_lump: float             # per unit (no reliquefaction, no nitrogen plant)
    paint_anodes_misc: float             # per unit
    battery_rate_per_kwh: float
    propulsion_rate_per_kw: float
    wheelhouse_accom_lump: float         # per tug
    tug_outfitting_lump: float           # per tug
    yard_margin_fraction: float          # fraction of barge + tug subtotal
    delivery_lump: float                 # per ATB unit
    currency: str = "USD"

    @classmethod
    def from_mapping(cls, data: Mapping) -> "BargeCostRates":
        names = {f.name for f in fields(cls)}
        unknown = sorted(set(data) - names)
        if unknown:
            raise ValueError(f"unknown barge cost rate key(s): {unknown}")
        missing = sorted(n for n in names - set(data) if n != "currency")
        if missing:
            raise ValueError(f"missing barge cost rate key(s): {missing}")
        return cls(**{k: (str(v) if k == "currency" else float(v)) for k, v in data.items()})


RatesLike = Union[BargeCostRates, Mapping, str, PathLike, None]


def _example_rates_text() -> str:
    """The shipped example: package data when installed, the repo's examples/ when run from source."""
    pkg = resources.files("vessel_designer").joinpath("data", EXAMPLE_RATES_FILE)
    if pkg.is_file():
        return pkg.read_text(encoding="utf-8")
    repo = Path(__file__).resolve().parents[3] / "examples" / EXAMPLE_RATES_FILE
    if repo.is_file():
        return repo.read_text(encoding="utf-8")
    raise FileNotFoundError(f"{EXAMPLE_RATES_FILE} not found; pass rates explicitly")


def parse_flat_yaml(text: str) -> Dict[str, str]:
    """``key: value`` lines, ``#`` comments and blank lines; anything else is an error."""
    return _call("barge_cost", "parse_flat_yaml", {"text": text}, Dict[str, str])


def load_rates(rates: RatesLike = None) -> BargeCostRates:
    """Rates from an object, a mapping, or a YAML path. None -> the illustrative example file."""
    if isinstance(rates, BargeCostRates):
        return rates
    if isinstance(rates, Mapping):
        return BargeCostRates.from_mapping(rates)
    text = _example_rates_text() if rates is None else Path(rates).read_text(encoding="utf-8")
    return BargeCostRates.from_mapping(parse_flat_yaml(text))


@dataclass
class BargeCostResult:
    barge_breakdown: Dict[str, float]
    tug_breakdown: Dict[str, float]
    barge_subtotal: float
    tug_subtotal: float
    yard_margin: float
    delivery: float
    unit_total: float                      # one ATB unit (barge + tug), delivered
    checks: List[Check] = field(default_factory=list)
    rates: Optional[BargeCostRates] = None   # the rates this result was priced with

    @property
    def ok(self) -> bool:
        return all_ok(self.checks)


def compute(inp: BargeCostInputs, rates: RatesLike = None) -> BargeCostResult:
    """Unit cost from the masses in ``inp`` and the ``rates`` you supply (see ``load_rates``)."""
    args = {"inputs": inp}
    if rates is None:
        pass                                  # the core's built-in example rates
    elif isinstance(rates, BargeCostRates):
        args["rates"] = rates
    elif isinstance(rates, Mapping):
        args["rates_mapping"] = dict(rates)
    else:
        args["rates_mapping"] = parse_flat_yaml(Path(rates).read_text(encoding="utf-8"))
    return _call("barge_cost", "compute", args, BargeCostResult)
