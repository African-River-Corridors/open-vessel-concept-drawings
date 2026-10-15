"""Aft arrangement — the self-propelled vessel's stern block, derived from its equipment.

WHY THIS MODULE EXISTS. The stern non-cargo length was a FRACTION of LOA
(``stern_void_fraction = 0.15``), typed once and then relied on by the capacity model, the
drawings and the material count. A fraction is not a design: it does not know whether three
thrusters fit across the beam, whether a 40 ft accommodation unit fits between the thruster
room and the cargo-area bulkhead, or how high the wheelhouse has to be to see over the
cargo. This module builds the stern block from the equipment that has to go in it and
returns its LENGTH, so the capacity model deducts a length it can justify.

The same block serves the gas tanker and the dry-bulk hopper. By default it holds one
40 ft battery unit, one 40 ft accommodation unit, three electric thrusters, and a
wheelhouse that lifts clear of the cargo.

ARRANGEMENT, stern -> bow:
    [ thruster / steering compartment ]
    [ 40 ft units, SIDE BY SIDE across the beam -- battery and accommodation ]
    [ wheelhouse, on deck, on a lifting column ]

The MIX of units differs by cargo: the dry-bulk vessel carries a second
BATTERY (it is the heavier hull and the longer leg), the gas vessel a second ACCOMMODATION
unit (a gas vessel is manned differently). Three units ABREAST either way, which costs beam
and not length -- 10.91 m of it.

A NARROW hull cannot do that: three 40 ft units abreast need 10.91 m, and the 8 m and 10 m
beam alternatives have neither. So the units overflow into a SECOND ROW fore-and-aft, which
lengthens the block and costs cargo. That is the real trade a narrow vessel makes, and it
is the reason this is modelled rather than gated: gating would have deleted the 8 m and
10 m columns of the design grid on a fact about ISO containers.

The wheelhouse is FORWARD of the containers and stows at DECK level, not on top of them.
That is the whole point of a lifting wheelhouse: stowed it is the lowest it can be, so it
does not drive the transit air draft, and raised its eye clears the cargo. Stowing it on
the container roof would add the container height to every bridge clearance.

The gas-code separation between this block and the cargo area is NOT here — it is a
cargo-code rule and lives in ``gas_separation.py``. Keeping them apart is deliberate: the
equipment does not change when the cargo does, and the separation does.

Typed dataclasses and one function; the calculation runs in the Rust core (`vessel_designer._native`). No I/O.
"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import List, Optional, Tuple

from .checks import Check, all_ok
from .._bridge import call as _call

# --- ISO 40 ft container envelope (ISO 668 series 1, 1AAA high-cube) ---
ISO40_LENGTH_M = 12.192
ISO40_WIDTH_M = 2.438
ISO40_HEIGHT_HC_M = 2.896
ISO40_HEIGHT_STD_M = 2.591

# Issue #180: the stern block's two propulsion arrangements.
PROPULSION_OPTIONS = ("battery_electric", "gas_fuelled")

# The Weichai gas genset catalogue the gas-fuelled block selects from, in selection order: one dict
# per set (model, engine, kWe, rpm, published engine envelope and mass, and the set envelope with
# the ESTIMATED alternator and base frame). Source: Weichai Power, "Weichai Medium Speed Marine
# Engine" (April 2023). The table lives in the Rust core (`GAS_GENSETS`).
GAS_GENSETS = _call("aft_arrangement", "genset_catalogue", {}, None)


@dataclass
class AftArrangementInputs:
    """What has to fit in the stern block, and how much room each item needs."""

    beam: float                              # moulded beam (m)
    depth: float                             # moulded depth = main deck above keel (m)

    # --- Propulsion (electric azimuth/tunnel units) ---
    # REQUESTED number of units (default three). It is a MAXIMUM, not a
    # promise: `compute` fits as many as the beam takes and reports the shortfall, because
    # a thruster size is set by power, not by beam, and three 2.6 m units do not go across
    # an 8 m hull at any clearance. Gating on it would delete half the design grid on the
    # strength of one estimate.
    n_thrusters: int = 3
    thruster_envelope_m: float = 2.60        # transverse envelope of one unit (m)
    thruster_clear_m: float = 0.60           # clear structure between units and to the shell
    thruster_room_length_m: float = 3.50     # steering flat / thruster compartment (m)

    # --- Containerised battery and accommodation (40 ft ISO, side by side) ---
    # Defaults are the GAS vessel's mix. `barge_capacity` swaps them for dry bulk.
    # None: the propulsion's default, 1 battery-electric, 0 gas-fuelled (issue #180).
    n_battery_units: Optional[int] = None
    n_accom_units: int = 2
    container_length_m: float = ISO40_LENGTH_M
    container_width_m: float = ISO40_WIDTH_M
    container_height_m: float = ISO40_HEIGHT_HC_M
    container_side_gap_m: float = 0.90       # walkway between and outboard of the units
    container_gap_m: float = 0.80             # clear between rows, fore and aft (m)
    container_foundation_m: float = 0.30     # seatings/stool under a deck-mounted unit (m)

    # --- Wheelhouse (lifting) ---
    wheelhouse_length_m: float = 3.60
    wheelhouse_width_m: float = 6.40         # wide cab
    wheelhouse_height_m: float = 3.40        # floor to roof (m) -- taller
    wheelhouse_gap_m: float = 0.30           # clear between the containers and the cab
    wheelhouse_eye_above_floor_m: float = 1.50
    # Height above keel the helmsman's eye must clear when raised. None = no cargo to see
    # over, so the wheelhouse stays in its lowered (transit) position.
    sightline_obstruction_m: Optional[float] = None
    sightline_margin_m: float = 0.50         # eye above the obstruction (m)
    wheelhouse_max_lift_m: float = 6.00      # stroke of the lifting column (m)

    # --- Propulsion (issue #180) ---
    # "battery_electric" (0.1.0, the default): battery units drive the thrusters.
    # "gas_fuelled": gas gensets below deck burn the cargo boil-off and drive the thrusters.
    # No battery; the accommodation units stand BESIDE the wheelhouse when the beam takes
    # them, and the engine room and gas valve unit (GVU) room sit below deck. The block is
    # then as long as the longer of the deck items and the rooms below.
    propulsion: str = "battery_electric"
    # Load (kWe) the gensets must carry with one set in reserve. When set, the genset and
    # count come from GAS_GENSETS (smallest installed power, n-1 sets carry the load) and
    # override the genset fields below.
    electrical_load_kwe: Optional[float] = None
    n_gensets: int = 2
    # Default set: Weichai CCFJ1200J-WMW (8WH20LCD1440-1). Envelope = published engine +
    # alternator 1.90 m long, 1.90 m wide + base frame 0.25 m (ESTIMATES, no vendor GA).
    genset_model: str = "CCFJ1200J-WMW"
    genset_kwe: float = 1200.0
    genset_length_m: float = 6.032
    genset_width_m: float = 1.90
    genset_height_m: float = 2.892
    genset_engine_mass_t: float = 12.5
    genset_clear_m: float = 1.0              # between sets and to the shell (ESTIMATE)
    engine_room_end_clear_m: float = 1.2     # maintenance space fore and aft (ESTIMATE)
    gvu_room_length_m: float = 2.0           # gas valve unit room (ESTIMATE)

    def __post_init__(self):
        if self.beam <= 0 or self.depth <= 0:
            raise ValueError("beam and depth must be positive")
        if self.n_thrusters < 1:
            raise ValueError("n_thrusters must be >= 1")
        if self.propulsion not in PROPULSION_OPTIONS:
            raise ValueError(f"propulsion must be one of {PROPULSION_OPTIONS}, "
                             f"got {self.propulsion!r}")


@dataclass
class AftArrangementResult:
    """The stern block, dimensioned. All x are metres FORWARD of the stern (x = 0)."""

    length_m: float                          # total block length the hull must give it

    # Longitudinal zones, (label, x_aft, x_fwd)
    zones: List[Tuple[str, float, float]] = field(default_factory=list)

    # Transverse thruster centres, metres from the centreline (+ = port)
    thruster_offsets_m: List[float] = field(default_factory=list)
    thruster_room_length_m: float = 0.0

    # Container block
    container_x0_m: float = 0.0
    container_x1_m: float = 0.0
    container_total_width_m: float = 0.0
    container_top_above_keel_m: float = 0.0
    # One label per unit, port to starboard, so the plan draws what the model counted
    # rather than assuming a fixed pair.
    container_labels: List[str] = field(default_factory=list)
    container_unit_width_m: float = 0.0
    container_side_gap_m: float = 0.0
    container_rows: int = 1
    container_per_row: int = 0
    container_row_length_m: float = 0.0
    container_row_gap_m: float = 0.0

    # Wheelhouse
    wheelhouse_x0_m: float = 0.0
    wheelhouse_x1_m: float = 0.0
    wheelhouse_floor_lowered_m: float = 0.0  # above keel
    wheelhouse_top_lowered_m: float = 0.0    # above keel — the TRANSIT air-draft top
    wheelhouse_floor_raised_m: float = 0.0
    wheelhouse_top_raised_m: float = 0.0     # above keel — the NAVIGATING top
    wheelhouse_lift_m: float = 0.0           # stroke actually used
    wheelhouse_width_m: float = 0.0
    wheelhouse_height_m: float = 0.0

    # The tallest fixed point of the block in the transit (lowered) condition
    block_top_lowered_m: float = 0.0

    checks: List[Check] = field(default_factory=list)
    warnings: List[str] = field(default_factory=list)

    # --- Issue #180: gas-fuelled stern (zeros/empty for battery_electric) ---
    propulsion: str = "battery_electric"
    container_layout: str = "rows"           # "rows" | "beside_wheelhouse"
    # Every unit's footprint: (label, x_aft, x_fwd, y_centre), y + = port.
    container_units: List[Tuple[str, float, float, float]] = field(default_factory=list)
    engine_room_x0_m: float = 0.0
    engine_room_x1_m: float = 0.0
    gvu_room_x0_m: float = 0.0
    gvu_room_x1_m: float = 0.0
    n_gensets: int = 0
    genset_model: str = ""
    genset_engine_model: str = ""
    genset_kwe: float = 0.0
    installed_kwe: float = 0.0
    genset_offsets_m: List[float] = field(default_factory=list)  # + = port
    genset_x0_m: float = 0.0
    genset_length_m: float = 0.0
    genset_width_m: float = 0.0
    genset_height_m: float = 0.0
    genset_engine_mass_t: float = 0.0        # published engine dry mass, all sets (t)
    below_deck_length_m: float = 0.0
    deck_length_m: float = 0.0

    @property
    def ok(self) -> bool:
        return all_ok(self.checks)


def compute(inp: AftArrangementInputs) -> AftArrangementResult:
    """Build the stern block from its equipment. Pure."""
    return _call("aft_arrangement", "compute", {"inputs": inp}, AftArrangementResult)
