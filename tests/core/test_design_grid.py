"""Design-grid sweep: golden base point, two tanks along, no tank-length cap, monotonicity."""
import math

from vessel_designer.core import design_grid
from vessel_designer.core.design_grid import sweep


def test_base_point_regression_anchor():
    """60 m barge x 12 m beam, 1.0 m standard clearance = 680 t.

    Set by the DERIVED ARRANGEMENT. A 60 m
    self-propelled gas vessel gives 21.9 m of its length to the stern block and the
    gas-code separation and 9.5 m to the bow rake and end bulkhead, leaving 28.6 m of
    cargo zone instead of 45.0 m. The drop is 48% -- far larger than on the 100 m
    reference vessel (-10%), because the stern block is a FIXED length and a short hull
    pays the same 21.9 m as a long one.

    Label: snapshot
    """
    g = sweep([60], [12.0])
    c = g.curve(60)[0]
    assert c.n_tanks_long == 2
    assert round(c.cargo_t) == 680


def test_every_cell_has_two_tanks_along_the_hull():
    """Two tanks along, as `gas_arrangement.fit` has (issue #175 removed the 28 m tank-length cap
    and with it the count that stepped up by length).

    Label: property
    """
    g = sweep([60, 100, 140], [8.0, 12.0, 16.0])
    assert {c.n_tanks_long for c in g.cells} == {2}
    assert not hasattr(design_grid, "n_tanks_for_length")
    assert not hasattr(design_grid, "MAX_TANK_LENGTH_M")
    assert not hasattr(g, "max_tank_length_m")


def test_a_tank_may_be_longer_than_28_m_but_not_more_than_seven_diameters():
    """Tank length is limited only by length over diameter and the hull.

    Label: property
    """
    long_tank = sweep([100], [14.0]).cells[0]
    assert long_tank.feasible and long_tank.tank_length_m > 28.0
    assert long_tank.slenderness <= 7.0
    thin = sweep([200], [11.0]).cells[0]
    assert not thin.feasible and "<= 7" in thin.reason
    g = sweep([40 + 10 * i for i in range(17)], [6.0, 8.0, 10.0, 12.0, 16.0, 20.0, 28.0])
    assert any(c.feasible for c in g.cells)
    assert all(c.slenderness <= 7.0 for c in g.cells if c.feasible)


def test_cargo_monotonic_in_length():
    """NARROWED. Monotonicity in BEAM no longer holds and should not be
    asserted: the tank outer diameter cap (`max_tank_od_m`) switches to more tanks abreast as
    the beam grows, and on a short hull two smaller tanks lose more to their heads than
    the extra beam wins -- 60 m x 16 m carries LESS than 60 m x 12 m. That is the design
    space, not a defect. Monotonicity in LENGTH does still hold, and is the invariant
    worth guarding, because length buys cargo zone one-for-one once the fixed end voids
    are paid. Narrow beams are out of this test's sample: below 11 m the three 40 ft units
    go into two rows, which lengthens the stern block, and short narrow hulls then have no
    cargo zone at all.

    Label: property
    """
    lengths = [70, 80, 90]
    beams = [12.0, 14.0]
    g = sweep(lengths, beams)
    for beam in beams:
        col = [next(c.cargo_t for c in g.curve(loa) if c.beam_m == beam)
               for loa in lengths]
        assert col == sorted(col), f"beam {beam} not monotonic in length: {col}"


def test_short_hulls_are_now_infeasible_for_gas():
    """REPLACES `test_all_cells_feasible_in_design_envelope`, and this is a
    FINDING rather than a test edit.

    The old test asserted that the whole reported envelope (45-90 m x 8-16 m) worked. It
    did, when the stern non-cargo zone was 15% of LOA -- 6.75 m on a 45 m hull. Once that
    zone is built from what actually goes in it (a 3.5 m thruster room, two 40 ft units,
    a wheelhouse and a 2.00 m gas-code separation = 21.9 m), a 45 m self-propelled gas
    vessel has no cargo area left at ANY beam in the envelope.

    So the old lower bound of the envelope does not survive the arrangement. Either
    the small vessels are pushed (a barge + separate tug, which moves the crew block off
    the cargo hull) or the envelope starts nearer 55-60 m. That is a design decision for the user.

    Label: snapshot
    """
    g = sweep([45, 60, 90], [8.0, 12.0, 16.0])
    by_loa = {}
    for c in g.cells:
        by_loa.setdefault(c.barge_length_m, []).append(c)
    assert all(not c.feasible for c in by_loa[45]), "45 m gas vessel should not fit"
    assert all(c.feasible for c in by_loa[60] if c.beam_m < 16.0), [c.reason for c in by_loa[60]]
    # Issue #175: with the 16 m diameter cap a 16 m beam is one 14 m tank, and two of those do not
    # fit in the 60 m hull (it was two tanks across under the old 12 m cap).
    assert [c.feasible for c in by_loa[60] if c.beam_m == 16.0] == [False]
    assert all(c.feasible for c in by_loa[90]), [c.reason for c in by_loa[90]]
