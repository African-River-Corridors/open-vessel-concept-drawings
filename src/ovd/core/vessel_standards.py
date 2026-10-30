"""Standard inland-vessel classes — a compact catalogue with a stable lookup API.

WHY THIS EXISTS. A fleet built to a STANDARD class, not a bespoke hull, lets a vessel built
or chartered elsewhere come onto a waterway without a custom build. This module holds those
classes as reference data plus a validity check, not an Inputs/Result/compute triple. The
class list is ``catalogue/vessel_classes.yaml``, the one source (issue #94): the core's list is
generated from it, and channel-design code looks a class up by id (``get_standard``). The ids of
the 12 classes this module used to type out (``CEMT_Va``, ``CN_II_motor``, ...) are aliases.

TWO CLASSIFICATION FAMILIES (of many in the catalogue), and they encode different philosophies:

  * CEMT/ITF-1992 (PIANC WG 141 Table 2.1, p15) and the Dutch 2011 update (Table 2.2,
    p16). European waterways are canalised and deep, so tonnage is bought with DRAUGHT
    on a narrow hull. CEMT'92 caps a single hull at 11.4 m beam; wider tonnage is a
    2-abreast pushed convoy at 22.8 m.
  * Chinese (WG 141 Table 2.3, p16; Chinese channel rules Appendix A.3, p162-164). Chinese
    inland waterways are shallow rivers, so tonnage is bought with BEAM: single motor
    vessels reach 14.8 m (2,000 t) and 16.2 m (3,000 t) at 2.6-3.2 m draught. Class
    numbering runs BACKWARDS from Europe's — Class I is the largest.

The Dutch 2011 update moved Europe the same way (VIa motor vessels at 13.5-17.0 m beam),
so CEMT'92 read alone will mislead. PIANC convened WG 179 to revise it.

WHY THE CHOICE IS LOAD-BEARING. Design channel depth follows draught, and design bend
radius follows LENGTH (R_min = k x LOA). Pairing barges IN LINE doubles LOA, so R_min
roughly doubles too — usually far more expensive on a meandering river than pairing ABREAST.

Dimensions are the CLASS definition, not a specific hull. Draught ranges are the class
range; a project's design draught is its own decision, and the channel depth follows from
it by the channel rules, not from this table.

The catalogue and its lookups run in the Rust core (`ovd._native`). No I/O.
"""

from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple

from .checks import Check
from .._bridge import call as _call


@dataclass(frozen=True)
class VesselStandard:
    """One standard inland-vessel or convoy class."""

    standard_id: str
    family: str                  # the scheme, e.g. "CEMT-1992", "RWS-2010", "CN-GB50139"
    class_name: str              # the class label: its catalogue id
    formation: str               # "motor_vessel" | "pushed_convoy" | "coupled_unit" | ...
    loa_m: Optional[Tuple[float, float]]   # (min, max) overall length; None when the class gives none
    beam_m: Optional[float]      # the largest beam of the class; None when it gives none
    draught_m: Optional[Tuple[float, float]]   # (min, max) draught range; None when none is given
    dwt_t: Optional[Tuple[float, float]]   # (min, max) deadweight; None when none is given
    n_abreast: int = 1           # barges side by side (sets beam)
    n_in_line: int = 1           # barges end to end (sets LOA, and so R_min)
    unit_beam_m: Optional[float] = None   # beam of ONE barge, where the class is a convoy
    min_bridge_clearance_m: Optional[float] = None   # the bottom of the class air-draft range
    #: No class sets a moulded depth (no source gives one; the depth stays typed). The field is
    #: kept so code that builds a standard keeps working.
    moulded_depth_m: Optional[float] = None
    source: str = ""
    cargo_modes: Tuple[str, ...] = ("dry_bulk", "liquid_gas")   # catalogue data; does not filter
    notes: str = ""
    air_draft_m: Optional[Tuple[float, float]] = None   # (min, max) air draft the class allows
    region: str = ""
    waters: str = ""
    confidence: str = ""         # "high" | "medium" | "low": confidence of the class envelope

    @property
    def loa_nominal_m(self) -> Optional[float]:
        return None if self.loa_m is None else 0.5 * (self.loa_m[0] + self.loa_m[1])

    @property
    def draught_nominal_m(self) -> Optional[float]:
        return None if self.draught_m is None else 0.5 * (self.draught_m[0] + self.draught_m[1])

    @property
    def dwt_nominal_t(self) -> Optional[float]:
        return None if self.dwt_t is None else 0.5 * (self.dwt_t[0] + self.dwt_t[1])

    def r_min_m(self, radius_multiple: float = 3.0) -> Optional[float]:
        """Design minimum bend radius at the LONG end of the class length range (None when the
        class gives no length).

        Sizing on the short end would under-call the radius for the same class.
        """
        return None if self.loa_m is None else radius_multiple * self.loa_m[1]


# --- the catalogue -------------------------------------------------------------------
# Every class of `catalogue/vessel_classes.yaml`, the one source (issue #94). The Rust core holds
# the list, generated from the YAML by `tools/gen_vessel_classes.py`; this dict is read from it, so
# there is no third hand-typed list. The ids of the 12 classes that used to be typed here
# (`CEMT_Va`, `CN_II_motor`, ...) are not keys, but `get_standard` still accepts them.
VESSEL_STANDARDS: Dict[str, VesselStandard] = {
    s.standard_id: s for s in _call("vessel_standards", "catalogue", {}, List[VesselStandard])}


def get_standard(standard_id: str) -> VesselStandard:
    """Look up a standard by id. The single accessor — never hard-code dimensions. The ids of the
    12 classes of 0.1.0 (``CEMT_Va``, ``CN_II_motor``, ...) still work."""
    return _call("vessel_standards", "get_standard", {"standard_id": standard_id}, VesselStandard)


def standards_for(cargo_mode: str) -> List[VesselStandard]:
    """0.1.0 listed the standards able to carry a cargo mode. Cargo is never a property of a Vessel
    Class, so every class is returned; the mode is still checked ('dry_bulk' or 'liquid_gas')."""
    return _call("vessel_standards", "standards_for", {"cargo_mode": cargo_mode}, List[VesselStandard])


def validate_catalogue(catalogue: Optional[Dict[str, VesselStandard]] = None) -> List[Check]:
    """Assert integrity of the fleet catalogue, as a list of Checks.

    Same contract as machinery.validate_catalogue: reference data earns a validity
    check, not an Inputs/Result/compute triple.
    """
    if catalogue is None:
        return _call("vessel_standards", "validate_catalogue", {"extra": None}, List[Check])
    return _call("vessel_standards", "validate_catalogue", {"catalogue": catalogue}, List[Check])
