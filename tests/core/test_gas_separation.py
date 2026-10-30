"""Gas-code longitudinal separation — golden, property and feasibility checks."""
from ovd.core.gas_separation import (COFFERDAM_MIN_M, OPENING_SETBACK_M,
                                          GasSeparationInputs, compute)


# ---------- golden ----------

def test_golden_defaults():
    """The 2,00 m opening setback governs, not the 0,60 m cofferdam: a deckhouse flush
    with the cargo area could carry no door and no openable window.

    Label: sourced - inland dangerous-goods classification rules Part 4 Ch. 4 Sec. 3 (July
        2022), cl. 3.3.7, 3.3.11, 3.5.3, 3.6.13 (as cited in core/gas_separation.py; clause
        numbers not yet checked against the ADN text)
    """
    r = compute(GasSeparationInputs())
    # Issue #381: the opening setback is measured from the cargo area, so the aft separation is the
    # end space plus the 2,00 m setback.
    assert abs(r.aft_separation_m - 2.20) < 1e-9
    # Issue #280: the end space defaults to the type G rule, 0.20 m (was 0.60 aft and 0.50 forward).
    assert abs(r.cofferdam_m - 0.20) < 1e-9
    assert abs(r.service_space_m - 2.00) < 1e-9
    assert abs(r.fwd_separation_m - 0.20) < 1e-9
    assert abs(r.total_length_taken_m - 2.40) < 1e-9
    assert r.ok


# ---------- property ----------

def test_aft_separation_never_below_either_rule():
    """Label: property
    """
    for cd in (0.0, 0.6, 1.5, 3.0):
        for os_ in (0.0, 2.0, 4.0):
            r = compute(GasSeparationInputs(cofferdam_m=cd, opening_setback_m=os_))
            assert r.aft_separation_m >= cd - 1e-9
            assert r.aft_separation_m >= os_ - 1e-9


def test_service_space_is_the_remainder():
    """Label: property
    """
    r = compute(GasSeparationInputs(cofferdam_m=0.9, opening_setback_m=3.0))
    assert abs(r.cofferdam_m + r.service_space_m - r.aft_separation_m) < 1e-9


# ---------- feasibility: the checks must fire ----------

def test_thin_aft_end_space_fails():
    """Issue #280: the 0.60 m cofferdam of the type C part (3.6.13) is no limit on a gas design. The
    type G space is 0.20 m between the tanks and the end bulkhead of the hold (3.5.3, ADN
    9.3.1.11.3(a)); 0.50 m for plane tank ends.

    Label: sourced - LR inland rules Pt 4 Ch 4 3.5.3; ADN 2025 9.3.1.11.3(a)
    """
    assert all(c.ok for c in compute(GasSeparationInputs(cofferdam_m=0.4)).checks)
    r = compute(GasSeparationInputs(cofferdam_m=0.1))
    assert any(c.name == "aft_end_bulkhead_space" and not c.ok for c in r.checks)


def test_short_opening_setback_fails():
    """Label: property
    """
    r = compute(GasSeparationInputs(cofferdam_m=0.6, opening_setback_m=1.0))
    assert any(c.name == "opening_setback" and not c.ok for c in r.checks)


def test_clearances_add_no_inland_floor_check():
    """Issue #215: the inland type-G floors are gone, so clearances below them fail nothing.

    Label: property
    """
    r = compute(GasSeparationInputs(side_clearance_m=0.05, double_bottom_m=0.05))
    assert [c.name for c in r.checks] == ["aft_end_bulkhead_space", "opening_setback", "fwd_end_bulkhead_space"]
    assert r.ok


def test_constants_are_the_cited_values():
    """Label: sourced - inland dangerous-goods classification rules Part 4 Ch. 4 Sec. 3 (July
        2022), cl. 3.3.7, 3.3.11, 3.6.13 (as cited in core/gas_separation.py; not yet checked
        against the ADN text)
    """
    assert COFFERDAM_MIN_M == 0.60 and OPENING_SETBACK_M == 2.00
