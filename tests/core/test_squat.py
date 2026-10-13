"""Test triad for squat: golden, property, feasibility.

Run with pytest: python -m pytest tests/core/test_squat.py
"""
import math

from vessel_designer.core.squat import (
    FNH_VALID_MAX,
    SquatInputs,
    compute,
    max_speed_for_ukc,
)

# An example case: CN II motor laden, in an example 4.80 m channel.
CASE = dict(draft_m=4.40, beam_m=14.8, lpp_m=87.3, block_coefficient=0.85,
           water_depth_m=4.80, channel_area_m2=209.5)


# --- golden ---------------------------------------------------------------------------

def test_golden_barrass_reproduces_its_own_formula():
    """S_max = Cb * S^0.81 * V^2.08 / 20, by hand.

    Label: sourced - Barrass II, S_max = Cb * S^0.81 * V^2.08 / 20 (Barrass; as quoted in
        core/squat.py), evaluated by hand in the test
    Parity: squat:compute/CN_II_4.8m_6kn
    """
    r = compute(SquatInputs(speed_through_water_kn=6.0, **CASE))
    s = (14.8 * 4.40) / 209.5
    expect = 0.85 * (s ** 0.81) * (6.0 ** 2.08) / 20.0
    assert abs(r.squat_barrass_m - expect) < 1e-9, (r.squat_barrass_m, expect)


def test_golden_zero_speed_is_zero_squat():
    """Label: property
    Parity: squat:compute/CN_II_4.8m_0kn
    """
    r = compute(SquatInputs(speed_through_water_kn=0.0, **CASE))
    assert r.squat_m == 0.0
    assert abs(r.dynamic_ukc_m - r.static_ukc_m) < 1e-12
    assert not r.grounds


def test_golden_the_finding_that_created_this_module():
    """resistance.py returned 9.2 kn for this vessel in this channel. At 9.2 kn the squat
    exceeds the 0.40 m under-keel clearance several times over — which is why speed here is
    set by squat and not by power.

    Label: snapshot
    Parity: squat:compute/CN_II_4.8m_9.2kn
    """
    r = compute(SquatInputs(speed_through_water_kn=9.2, **CASE))
    assert abs(r.static_ukc_m - 0.40) < 1e-9
    assert r.squat_m > 1.0, r.squat_m
    assert r.grounds
    assert not [c for c in r.checks if c.name == "keel_clears_bed"][0].ok


# --- property -------------------------------------------------------------------------

def test_property_squat_rises_monotonically_with_speed():
    """Label: property
    Parity: squat:compute/CN_II_4.8m_1kn, squat:compute/CN_II_4.8m_2kn,
        squat:compute/CN_II_4.8m_4kn, squat:compute/CN_II_4.8m_8kn
    """
    prev = -1.0
    for v in (0.0, 1.0, 2.0, 4.0, 6.0, 8.0):
        s = compute(SquatInputs(speed_through_water_kn=v, **CASE)).squat_m
        assert s >= prev, v
        prev = s


def test_property_squat_rises_with_blockage():
    """A narrower channel squats the same ship deeper at the same speed.

    Label: property
    Parity: squat:compute/CN_II_4.8m_5kn, squat:compute/CN_II_4.8m_5kn_area_400,
        squat:compute/CN_II_4.8m_5kn_area_300, squat:compute/CN_II_4.8m_5kn_area_250
    """
    prev = 0.0
    for area in (400.0, 300.0, 250.0, 209.5):          # decreasing = more blockage
        a = dict(CASE); a["channel_area_m2"] = area
        s = compute(SquatInputs(speed_through_water_kn=5.0, **a)).squat_m
        assert s > prev, area
        prev = s


def test_property_deeper_water_squats_less():
    """Label: property
    Parity: squat:compute/CN_II_depth_5.28_5kn, squat:compute/CN_II_depth_6_5kn,
        squat:compute/CN_II_depth_8_5kn
    """
    prev = None
    for h in (4.8, 5.28, 6.0, 8.0):
        a = dict(CASE); a["water_depth_m"] = h
        r = compute(SquatInputs(speed_through_water_kn=5.0, **a))
        if prev is not None:
            assert r.squat_m <= prev + 1e-9, h
        prev = r.squat_m


def test_property_dynamic_ukc_is_static_minus_squat():
    """Label: property
    Parity: squat:compute/CN_II_4.8m_3kn, squat:compute/CN_II_4.8m_5kn,
        squat:compute/CN_II_4.8m_7kn
    """
    for v in (0.0, 3.0, 5.0, 7.0):
        r = compute(SquatInputs(speed_through_water_kn=v, **CASE))
        assert abs(r.dynamic_ukc_m - (r.static_ukc_m - r.squat_m)) < 1e-12


def test_property_max_speed_solver_lands_on_the_margin():
    """Label: property
    Parity: squat:max_speed_for_ukc/depth5.28_area235.5_margin0.15,
        squat:compute/depth5.28_area235.5_at_solver,
        squat:compute/depth5.28_area235.5_solver_plus_0.1
    """
    v = max_speed_for_ukc(water_depth_m=5.28, channel_area_m2=235.5,
                          draft_m=4.40, beam_m=14.8, lpp_m=87.3, safety_margin_m=0.15)
    r = compute(SquatInputs(speed_through_water_kn=v, draft_m=4.40, beam_m=14.8, lpp_m=87.3,
                            water_depth_m=5.28, channel_area_m2=235.5))
    assert r.dynamic_ukc_m >= 0.15 - 1e-3, r.dynamic_ukc_m
    r2 = compute(SquatInputs(speed_through_water_kn=v + 0.1, draft_m=4.40, beam_m=14.8,
                             lpp_m=87.3, water_depth_m=5.28, channel_area_m2=235.5))
    assert r2.dynamic_ukc_m < 0.15, "solver should be AT the margin, not below it"


def test_property_deeper_channel_buys_speed():
    """The deeper-channel argument, as an invariant: 0.48 m more water is materially more speed.

    Label: property
    Parity: squat:max_speed_for_ukc/depth4.8_area209.5,
        squat:max_speed_for_ukc/depth5.28_area235.5
    """
    v48 = max_speed_for_ukc(water_depth_m=4.80, channel_area_m2=209.5,
                            draft_m=4.40, beam_m=14.8, lpp_m=87.3)
    v53 = max_speed_for_ukc(water_depth_m=5.28, channel_area_m2=235.5,
                            draft_m=4.40, beam_m=14.8, lpp_m=87.3)
    assert v53 > v48 * 1.4, (v48, v53)


# --- feasibility: the checks must fire when they should ---------------------------------

def test_feasibility_grounding_is_reported_not_returned_silently():
    """Label: property
    Parity: squat:compute/CN_II_4.8m_8kn
    """
    r = compute(SquatInputs(speed_through_water_kn=8.0, **CASE))
    assert r.grounds
    assert r.dynamic_ukc_m < 0
    assert not r.ok, "a grounding must fail the result, not merely warn"


def test_feasibility_supercritical_froude_is_flagged():
    """Label: property
    Parity: squat:compute/CN_II_depth_3_9kn
    """
    a = dict(CASE); a["water_depth_m"] = 3.0
    r = compute(SquatInputs(speed_through_water_kn=9.0, **a))
    assert r.froude_depth > FNH_VALID_MAX
    assert not [c for c in r.checks if c.name == "froude_depth_subcritical"][0].ok


def test_feasibility_open_water_says_so():
    """No channel area means blockage 0, Barrass 0, and ICORELS carrying the answer alone.

    Label: property
    Parity: squat:compute/CN_II_open_water_8kn
    """
    a = dict(CASE); a["channel_area_m2"] = 0.0
    r = compute(SquatInputs(speed_through_water_kn=8.0, **a))
    assert r.blockage_ratio == 0.0
    assert r.squat_barrass_m == 0.0
    assert any("OPEN-WATER" in w for w in r.warnings)
    assert r.method == "ICORELS"


def test_feasibility_method_disagreement_is_surfaced():
    """The two forms differ; the result must say by how much rather than pick one quietly.

    Label: property
    Parity: squat:compute/CN_II_4.8m_6kn
    """
    r = compute(SquatInputs(speed_through_water_kn=6.0, **CASE))
    c = [c for c in r.checks if c.name == "methods_agree_within_50pct"]
    assert c, "the comparison check must exist whenever both methods evaluated"
    assert r.squat_icorels_m is not None
    assert r.method in ("Barrass II", "ICORELS")
    assert r.squat_m == max(r.squat_barrass_m, r.squat_icorels_m)
