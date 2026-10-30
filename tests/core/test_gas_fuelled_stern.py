"""Gas-fuelled stern block (issue #180): Weichai gas gensets below deck, accommodation beside the
wheelhouse, thrusters aft. The battery-electric block of 0.1.0 is unchanged.
"""
import dataclasses

import pytest

from ovd.core.aft_arrangement import (PROPULSION_OPTIONS, AftArrangementInputs,
                                                  compute, GAS_GENSETS)
from ovd.core.barge_capacity import BargeCapacityInputs
from ovd.core.barge_capacity import compute as capacity


def _gas(**kw):
    kw.setdefault("beam", 26.0)
    kw.setdefault("depth", 7.0)
    return compute(AftArrangementInputs(propulsion="gas_fuelled", **kw))


def test_propulsion_options():
    """Label: property"""
    assert PROPULSION_OPTIONS == ("battery_electric", "gas_fuelled")
    with pytest.raises(ValueError, match="propulsion"):
        AftArrangementInputs(beam=26.0, depth=7.0, propulsion="diesel")


def test_catalogue_is_the_published_weichai_table():
    """Label: sourced - Weichai Power, "Weichai Medium Speed Marine Engine" (April 2023),
    slides 12 and 15 (engine dimensions and mass) and 30 and 31 (generating-set list)."""
    cat = GAS_GENSETS
    assert [g["kwe"] for g in cat] == [900, 1050, 1200, 1400, 1600, 1800, 2000, 2200]
    assert cat[2]["genset_model"] == "CCFJ1200J-WMW" and cat[2]["engine_model"] == "8WH20LCD1440-1"
    assert (cat[2]["engine_length_m"], cat[2]["engine_mass_t"]) == (4.132, 12.5)
    # The dataclass default set is catalogue entry 2, with the same envelope.
    d = AftArrangementInputs(beam=26.0, depth=7.0)
    assert (d.genset_model, d.genset_kwe) == (cat[2]["genset_model"], cat[2]["kwe"])
    assert d.genset_length_m == pytest.approx(cat[2]["length_m"])
    assert d.genset_height_m == pytest.approx(cat[2]["height_m"])


def test_140_m_lng_barge_block():
    """Label: sourced - hand sum. 26 m beam, 1910 kWe -> 3 x 1050 kWe (2 carry the load).
    Below deck 3.5 + (3.532 + 1.90 + 2.4) + 2.0 = 13.332 m; deck 3.5 + 0.3 + 12.192 = 15.992 m."""
    r = _gas(electrical_load_kwe=1910.0)
    assert (r.n_gensets, r.genset_kwe, r.installed_kwe) == (3, 1050.0, 3150.0)
    assert r.container_layout == "beside_wheelhouse"
    assert r.below_deck_length_m == pytest.approx(13.332)
    assert r.length_m == pytest.approx(15.992)
    assert r.wheelhouse_x1_m == pytest.approx(r.container_x1_m)
    assert not any("BATTERY" in lab for lab in r.container_labels)
    assert [round(u[3], 3) for u in r.container_units] == [5.319, -5.319]
    assert r.ok


def test_gas_block_is_shorter_than_the_battery_block_on_a_wide_hull():
    """Label: property"""
    gas = _gas(electrical_load_kwe=1910.0)
    be = compute(AftArrangementInputs(beam=26.0, depth=7.0))
    assert gas.length_m < be.length_m


def test_no_battery_and_length_is_the_longer_layer():
    """Label: property"""
    for beam in (10.0, 14.8, 20.0, 26.0):
        r = _gas(beam=beam, depth=6.0, electrical_load_kwe=1400.0)
        assert r.length_m == max(r.deck_length_m, r.below_deck_length_m)
        assert not any("BATTERY" in lab for lab in r.container_labels)
    with pytest.raises(ValueError, match="battery"):
        _gas(n_battery_units=1)


def test_battery_electric_default_is_unchanged():
    """Label: property"""
    r = compute(AftArrangementInputs(beam=14.8, depth=5.4))
    assert r.length_m == pytest.approx(19.892)
    assert r.container_labels == ["BATTERY 40 FT", "ACCOM 40 FT", "ACCOM 40 FT"]
    assert (r.propulsion, r.n_gensets, r.container_layout) == ("battery_electric", 0, "rows")


def test_barge_capacity_passes_propulsion_through():
    """Label: property"""
    base = BargeCapacityInputs(loa=140.0, beam=26.0, depth=7.0, cargo_type="LNG_ethane")
    be = capacity(base)
    gas = capacity(dataclasses.replace(base, propulsion="gas_fuelled", electrical_load_kwe=1910.0))
    a = gas.aft_arrangement
    assert a.propulsion == "gas_fuelled" and a.n_gensets == 3
    assert not any("BATTERY" in lab for lab in a.container_labels)
    assert be.aft_arrangement.propulsion == "battery_electric"
    # The shorter block leaves the cargo zone no shorter.
    assert gas.stern_void_length_m < be.stern_void_length_m
    assert gas.cargo_volume_m3 >= be.cargo_volume_m3
    with pytest.raises(ValueError, match="disagrees"):
        capacity(dataclasses.replace(base, propulsion="gas_fuelled",
                                     aft=AftArrangementInputs(beam=26.0, depth=7.0)))
