"""The tank rulesets. IGC/IGF behaviour unchanged; the ADN type G ruleset was removed (issue #175)."""
from vessel_designer.core.protective_location import compute_required_clearances
from vessel_designer.core.barge_capacity import BargeCapacityInputs, compute


def test_igc_2g_unchanged():
    """Label: snapshot
    """
    assert compute_required_clearances("IGC_2G", 14.8, 1.0).cs_required == 1.0
    assert compute_required_clearances("IGC_2G", 14.8, 2.0).cb_required == 2.0
    assert compute_required_clearances("IGC_2G", 14.8, 0.8).cb_required == 14.8 / 15.0


def test_adn_g_is_no_longer_a_vessel_type():
    """Issue #175: the engine applies IGC rules only.

    Label: property
    """
    import pytest
    from vessel_designer.core import protective_location
    assert [m.name for m in protective_location.VesselType] == [
        "IGF", "IGC_1G", "IGC_2G", "IGC_3G", "IGC_2G_EXACT"]
    with pytest.raises(ValueError, match="'ADN_G' is not a valid VesselType"):
        compute_required_clearances("ADN_G", 18.0, 1.0)
    assert not hasattr(protective_location, "ADN_G_SIDE_M")


def test_d_vc_exact_bands():
    """Label: sourced - IGC Code 2.4.1.1 d(Vc) bands (as cited in core/protective_location.py),
        evaluated by hand for 800, 3000, 5000, 10000 and 40000 m3
    """
    from vessel_designer.core.protective_location import d_vc_exact
    assert d_vc_exact(800) == 0.8
    assert abs(d_vc_exact(3000) - 0.9) < 1e-12 and abs(d_vc_exact(5000) - 1.0) < 1e-12
    assert abs(d_vc_exact(10000) - 1.2) < 1e-12 and d_vc_exact(40000) == 2.0


def test_igc_exact_measures_to_steel_and_uses_each_tank():
    """Label: property
    """
    from vessel_designer.core.protective_location import d_vc_exact
    r = compute(BargeCapacityInputs(loa=90.0, beam=14.8, depth=6.0, cargo_type="LNG",
                                    n_tanks_wide=1, vessel_type="IGC_2G_EXACT"))
    per_tank = r.containment_inner_volume_m3 / r.n_tanks_total
    # side clearance to the insulation face = d(per tank) - 0.35 m insulation
    assert abs(r.side_clearance_m - (d_vc_exact(per_tank) - 0.35)) < 0.02
    old = compute(BargeCapacityInputs(loa=90.0, beam=14.8, depth=6.0, cargo_type="LNG",
                                      n_tanks_wide=1, vessel_type="IGC_2G"))
    assert r.tank_outer_diameter_m > old.tank_outer_diameter_m
