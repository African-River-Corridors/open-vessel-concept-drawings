"""Gas-barge intact stability. Independent oracles: closed-form box formulas, not the module."""
import math

import pytest

from ovd.core.barge_capacity import BargeCapacityInputs, compute as capacity
from ovd.core.gas_arrangement import fit
from ovd.core.gas_stability import assess, box_gz, criteria_set_for, segment_fill


@pytest.mark.parametrize("deg", [2, 5, 10, 15, 20])
def test_gz_matches_the_wall_sided_formula_before_deck_edge(deg):
    """Label: sourced - closed-form wall-sided GZ of a box section, sin(phi) (GM + BM/2 tan^2(phi))
        (standard naval architecture), evaluated in the test
    """
    B, D, T, KG = 14.8, 6.0, 3.0, 4.0               # deck edge immerses at 22.1 deg
    bm = B * B / (12 * T)
    gm = T / 2 + bm - KG
    phi = math.radians(deg)
    assert box_gz(B, D, T, KG, deg) == pytest.approx(math.sin(phi) * (gm + bm / 2 * math.tan(phi) ** 2), abs=1e-6)


def test_gz_is_antisymmetric_and_falls_past_the_deck_edge():
    """Label: property
    """
    B, D, T, KG = 14.8, 4.0, 3.0, 3.5
    assert box_gz(B, D, T, KG, -12) == pytest.approx(-box_gz(B, D, T, KG, 12))
    wall_sided_40 = math.sin(math.radians(40)) * (T / 2 + B * B / (12 * T) - KG + B * B / (24 * T) * math.tan(math.radians(40)) ** 2)
    assert box_gz(B, D, T, KG, 40) < wall_sided_40    # deck immersed: less than the wall-sided value


def test_half_full_cylinder_segment():
    """Label: sourced - hand calculation: a half-full circular segment has height r and its
        centroid 4r/(3 pi) from the centre
    """
    h, ybar = segment_fill(2.0, 0.5)
    assert h == pytest.approx(2.0)
    assert ybar == pytest.approx(2.0 - 4 * 2.0 / (3 * math.pi))


def only_the_light_angle_may_fail(a):
    """Issue #498: in the Light condition (no ballast) a failure where the angle of maximum GZ is the
    only failing criterion is a warning of `fit`, not a failure. Every other criterion of every
    condition holds."""
    return all(c.ok or (c.name == "Light" and [k.name for k in c.criteria if not k.ok] == ["Angle of max GZ"])
               for c in a.conditions)


def test_cn_ii_passes_the_is_code():
    """Changed in issue #237. Under the per-tank rule (IGC 2.4.1.1), the CN II hull with the 16 m
    default tank cap carries 2,943 t in one 13.7 m tank and FAILS the Laden angle of maximum GZ
    (21 degrees, limit 25); see the next test. This test is the CN II hull at the 12 m cap this
    test was written for, which passes.

    Label: snapshot
    """
    f = fit(90.0, 14.8, "LPG_propane", 4.4, depths=[6.5], ruleset="IGC_2G", fill_ratio=0.95,
            max_tank_od_m=12.0)
    a = assess(f.engine_result, f.payload_t, criteria_set="IS_CODE_2008")
    assert only_the_light_angle_may_fail(a)
    assert {c.name for c in a.conditions} == {"Laden", "Half laden", "Light"}


def test_cn_ii_at_the_default_16_m_cap_passes_stability_because_the_search_gates_on_it():
    """Issue #237 finding: with the smaller side clearance of the per-tank rule, the search put more
    cargo in one wider tank and the laden condition was tender. Since issue #264 the search keeps
    only arrangements that pass stability, so the design it returns is stable in every condition.

    Label: property
    """
    f = fit(90.0, 14.8, "LPG_propane", 4.4, depths=[6.5], ruleset="IGC_2G", fill_ratio=0.95)
    assert f.feasible and f.failed_checks == []
    a = assess(f.engine_result, f.payload_t, criteria_set="IS_CODE_2008")
    assert only_the_light_angle_may_fail(a)


def test_a_hull_where_nothing_passes_has_a_failing_assessment():
    """The check must be able to fail: on a 60 x 20 x 3.5 m self-propelled hull every arrangement
    fails the stability of more than the light condition. `fit` says so (`feasible` false, the failed
    checks named) and the closest candidate it returns fails its own assessment. (Before issue #498
    this was a 60 x 10 x 6 m hull whose only failure was the angle of maximum GZ in the light
    condition; that is a warning now, see the next test.)

    Label: property
    """
    f = fit(60.0, 20.0, "LPG_propane", 3.2, depths=[3.5], ruleset="IGC_2G", fill_ratio=0.98)
    assert not f.feasible
    assert f.failed_checks and all(m.startswith("Stability \u2014 ") for m in f.failed_checks)
    a = assess(f.engine_result, f.payload_t, criteria_set="IS_CODE_2008")
    assert not a.ok
    assert not only_the_light_angle_may_fail(a)


def test_a_light_condition_that_fails_only_the_angle_of_max_gz_is_feasible_with_a_failing_assessment():
    """Issue #498: 60 x 10 x 6 m. The Light condition (no ballast) fails IS Code 2.2.3, the angle of
    maximum GZ, and nothing else. `fit` calls the arrangement feasible; the assessment, as in 0.1.0,
    still says the Light condition does not pass.

    Label: property
    """
    f = fit(60.0, 10.0, "LPG_propane", 5.7, depths=[6.0], ruleset="IGC_2G", fill_ratio=0.98)
    assert f.feasible and f.failed_checks == []
    a = assess(f.engine_result, f.payload_t, criteria_set="IS_CODE_2008")
    assert not a.ok and only_the_light_angle_may_fail(a)
    light = next(c for c in a.conditions if c.name == "Light")
    assert [k.name for k in light.criteria if not k.ok] == ["Angle of max GZ"]


def test_the_igc_rule_set_uses_the_is_code_and_the_adn_set_is_gone():
    """Issue #175 removed the ADN criteria set with the ADN_G ruleset. Issue #237: one rule set.

    Label: property
    """
    assert criteria_set_for("IGC_2G") == "IS_CODE_2008"
    f = fit(110.0, 11.4, "LNG", 3.0, depths=[4.5], self_propelled=False)
    with pytest.raises(ValueError, match="unknown criteria set 'ADN_9_3_1_14_2'"):
        assess(f.engine_result, f.payload_t, criteria_set="ADN_9_3_1_14_2")


def test_bulk_result_is_refused():
    """Label: property
    """
    r = capacity(BargeCapacityInputs(loa=60, beam=11, depth=4, cargo_type="bauxite"))
    with pytest.raises(ValueError):
        assess(r, 1000.0)
