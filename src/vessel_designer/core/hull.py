"""Box barge hull geometry with bottom rakes at bow and stern.

Side view of hull:
                          |  bow
                          |
         -----------------+  <- deck
        /                 |
       /                  |  <- bow rake (bottom slopes up)
      /                   |
-----+--------------------+  <- keel (flat bottom)
     <--- rake_length --->

The rake is a bottom rake: the hull sides remain vertical, but the bottom
slopes upward at the bow (and optionally stern). This is typical for
pushed/ATB barges on inland waterways.
"""

from dataclasses import dataclass
from .._bridge import call as _call


@dataclass
class HullGeometry:
    """Box barge hull with bottom rakes."""

    loa: float              # Length overall (m)
    beam: float             # Moulded beam (m)
    depth: float            # Moulded depth (m)
    bow_rake_length: float  # Horizontal length of bow rake (m)
    bow_rake_rise: float    # Vertical rise of bow rake (m)
    stern_rake_length: float  # Horizontal length of stern rake (m)
    stern_rake_rise: float  # Vertical rise of stern rake (m)

    def _q(self, draft: float) -> dict:
        """Every quantity of this hull at ``draft``, from the Rust core."""
        return _call("hull", "quantities", {"hull": self, "draft": draft})

    @property
    def parallel_length(self) -> float:
        """Length of the parallel midbody (flat-bottom section)."""
        return self._q(0.0)["parallel_length"]


    def displaced_volume(self, draft: float) -> float:
        """Total displaced volume at given draft (m^3)."""
        return self._q(draft)["displaced_volume"]

    def waterplane_area(self, draft: float) -> float:
        """Waterplane area at given draft (m^2)."""
        return self._q(draft)["waterplane_area"]

    def waterplane_length(self, draft: float) -> float:
        """Effective waterplane length at given draft (m)."""
        return self._q(draft)["waterplane_length"]

    def waterplane_inertia_transverse(self, draft: float) -> float:
        """Second moment of waterplane area about the centerline (m^4).

        For a rectangular waterplane: I_T = (1/12) * L_wp * B^3
        This is exact for a box barge since the waterplane is always
        full beam (rakes are bottom rakes, sides are vertical).
        """
        return self._q(draft)["waterplane_inertia_transverse"]

    def waterplane_inertia_longitudinal(self, draft: float) -> float:
        """Second moment of waterplane area about the transverse center (m^4).

        For longitudinal stability and trim calculations.
        I_L = (1/12) * B * L_wp^3 (approximate, treating waterplane as rectangle)

        More precisely, we need the second moment about the center of flotation.
        For a symmetric box barge (equal rakes), the center of flotation is at
        midships. For asymmetric rakes, it shifts.
        """
        return self._q(draft)["waterplane_inertia_longitudinal"]

    def KB(self, draft: float) -> float:
        """Vertical position of center of buoyancy above keel (m).

        For the parallel midbody, KB = T/2.
        For rake sections, the centroid is lower because there is less
        volume near the surface. We compute by integrating.
        """
        return self._q(draft)["KB"]

    def LCB(self, draft: float) -> float:
        """Longitudinal center of buoyancy measured from the stern (AP) (m).

        Uses the convention: x=0 at stern, x=LOA at bow.
        """
        return self._q(draft)["LCB"]

    def BM_transverse(self, draft: float) -> float:
        """Transverse metacentric radius BM = I_T / V (m)."""
        return self._q(draft)["BM_transverse"]

    def BM_longitudinal(self, draft: float) -> float:
        """Longitudinal metacentric radius BM_L = I_L / V (m)."""
        return self._q(draft)["BM_longitudinal"]

    # --- Surface areas for weight estimation ---

    def bottom_area(self) -> float:
        """Flat bottom plating area (m^2)."""
        return self._q(0.0)["bottom_area"]

    def side_area(self) -> float:
        """Total side plating area, both sides (m^2).

        Sides are vertical for the full LOA.
        """
        return self._q(0.0)["side_area"]

    def deck_area(self) -> float:
        """Deck plating area (m^2).

        Deck covers the full LOA x beam.
        """
        return self._q(0.0)["deck_area"]

    def bow_rake_area(self) -> float:
        """Bow rake plating area (m^2).

        The rake is a sloped plate: length along slope = sqrt(rake_length^2 + rake_rise^2)
        Width = beam
        """
        return self._q(0.0)["bow_rake_area"]

    def stern_rake_area(self) -> float:
        """Stern rake plating area (m^2)."""
        return self._q(0.0)["stern_rake_area"]

    def total_hull_surface_area(self) -> float:
        """Total hull shell plating area (m^2)."""
        return self._q(0.0)["total_hull_surface_area"]
