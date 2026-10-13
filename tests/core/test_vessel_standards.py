"""Test triad for vessel_standards: golden, property, feasibility (ported from the prior engine)."""

from vessel_designer.core.vessel_standards import (
    VESSEL_STANDARDS,
    VesselStandard,
    get_standard,
    standards_for,
    validate_catalogue,
)


# --- golden: the class definitions, straight from WG 141 ------------------------------

def test_golden_cemt_via():
    """CEMT VIa: 2 Va abreast (WG 141 Table 2.1).

    Label: sourced - PIANC WG 141 Table 2.1, CEMT class VIa (as cited in core/vessel_standards.py)
    Parity: vessel_standards:get_standard/CEMT_VIa
    """
    s = get_standard("CEMT_VIa")
    assert s.loa_m == (95.0, 110.0), s.loa_m
    assert s.beam_m == 22.8, s.beam_m
    assert s.dwt_t == (3200.0, 6000.0), s.dwt_t
    assert s.n_abreast == 2 and s.unit_beam_m == 11.4
    # 2 x Va abreast: the overall beam must be exactly twice the unit beam
    assert abs(s.beam_m - 2 * s.unit_beam_m) < 1e-9


def test_golden_chinese_classes():
    """Chinese classes buy tonnage with beam, not draught (WG 141 Table 2.3).

    Label: sourced - PIANC WG 141 Table 2.3, Chinese classes I and II (as cited in
        core/vessel_standards.py)
    Parity: vessel_standards:get_standard/CN_II_motor, vessel_standards:get_standard/CN_I_motor
    """
    i, ii = get_standard("CN_I_motor"), get_standard("CN_II_motor")
    assert (i.beam_m, i.draught_m[0], i.dwt_t[0]) == (16.2, 3.2, 3000.0)
    assert (ii.beam_m, ii.draught_m[0], ii.dwt_t[0]) == (14.8, 2.6, 2000.0)
    # Chinese numbering runs BACKWARDS: Class I is the larger of the two.
    assert i.dwt_nominal_t > ii.dwt_nominal_t


def test_golden_beam_over_draught_tradeoff():
    """At comparable tonnage the Chinese hull is wider and shallower than the European.

    CN Class II (2,000 t) vs CEMT Va (1,500-3,000 t) — this is the whole reason the
    classification family matters for a shallow river.

    Label: sourced - PIANC WG 141 Tables 2.1 and 2.3, CN class II and CEMT Va (as cited in
        core/vessel_standards.py)
    Parity: vessel_standards:get_standard/CN_II_motor, vessel_standards:get_standard/CEMT_Va
    """
    cn, eu = get_standard("CN_II_motor"), get_standard("CEMT_Va")
    assert cn.beam_m > eu.beam_m                      # 14.8 > 11.4
    assert cn.draught_m[1] < eu.draught_m[1]          # 2.6 < 4.5


# --- property: invariants that must hold for every entry ------------------------------

def test_property_ranges_and_dimensions():
    """Label: property
    Parity: vessel_standards:validate_catalogue/built_in
    """
    for s in VESSEL_STANDARDS.values():
        assert s.loa_m[0] <= s.loa_m[1]
        assert s.draught_m[0] <= s.draught_m[1]
        assert s.dwt_t[0] <= s.dwt_t[1]
        assert s.beam_m > 0 and s.loa_m[0] > 0
        assert s.loa_m[0] <= s.loa_nominal_m <= s.loa_m[1]
        assert s.dwt_t[0] <= s.dwt_nominal_t <= s.dwt_t[1]


def test_property_r_min_scales_with_length_not_beam():
    """R_min follows LOA only. This is why pairing IN LINE is so expensive here and
    pairing ABREAST is not.

    Label: property
    Parity: vessel_standards:get_standard/CEMT_VIa, vessel_standards:get_standard/CEMT_Va,
        vessel_standards:get_standard/CEMT_Vb
    """
    in_line = get_standard("CEMT_Vb")        # 2 in line, 185 m
    abreast = get_standard("CEMT_VIa")       # 2 abreast, 110 m
    assert in_line.beam_m < abreast.beam_m           # narrower...
    assert in_line.r_min_m() > abreast.r_min_m()     # ...but needs a far bigger radius
    assert in_line.r_min_m(3.0) == 3.0 * 185.0

    # and R_min is sized on the LONG end of the class, never the short end
    va = get_standard("CEMT_Va")
    assert va.r_min_m(3.0) == 3.0 * va.loa_m[1]


def test_unknown_standard_id_raises_keyerror():
    """Label: property
    Parity: vessel_standards:get_standard/error/unknown_id
    """
    try:
        get_standard("no_such_class")
    except KeyError as exc:
        assert "unknown vessel standard" in str(exc)
    else:
        raise AssertionError("get_standard should raise on an unknown id")


def test_property_cargo_modes():
    """Label: property
    Parity: vessel_standards:standards_for/dry_bulk, vessel_standards:standards_for/liquid_gas,
        vessel_standards:standards_for/error/bananas
    """
    for mode in ("dry_bulk", "liquid_gas"):
        got = standards_for(mode)
        assert got, mode
        assert all(mode in s.cargo_modes for s in got)
    try:
        standards_for("bananas")
    except ValueError:
        pass
    else:
        raise AssertionError("standards_for should reject an unknown cargo mode")


# --- feasibility: the checks fire when they should ------------------------------------

def test_feasibility_catalogue_is_clean():
    """Label: property
    Parity: vessel_standards:validate_catalogue/built_in
    """
    for c in validate_catalogue():
        assert c.ok, f"{c.name}: {c.message}"


def test_feasibility_checks_catch_a_bad_entry():
    """A convoy whose beam does not reconcile with n_abreast must be caught.

    Label: property
    Parity: vessel_standards:validate_catalogue/with_broken_convoy
    """
    bad = dict(VESSEL_STANDARDS)
    bad["BROKEN"] = VesselStandard(
        standard_id="BROKEN", family="CEMT", class_name="X", formation="pushed_convoy",
        loa_m=(100.0, 110.0), beam_m=30.0, draught_m=(2.5, 3.0), dwt_t=(1000.0, 2000.0),
        n_abreast=2, unit_beam_m=11.4, source="fabricated for the test",
    )
    names = {c.name: c for c in validate_catalogue(bad)}
    assert not names["convoy_beam_consistent"].ok
    assert names["unique_ids"].ok           # the other checks still pass


def test_feasibility_inverted_range_is_caught():
    """Label: property
    Parity: vessel_standards:validate_catalogue/with_inverted_range
    """
    bad = dict(VESSEL_STANDARDS)
    bad["BACKWARDS"] = VesselStandard(
        standard_id="BACKWARDS", family="CN", class_name="X", formation="motor_vessel",
        loa_m=(120.0, 90.0), beam_m=14.0, draught_m=(2.5, 3.0), dwt_t=(1000.0, 2000.0),
        source="fabricated for the test",
    )
    names = {c.name: c for c in validate_catalogue(bad)}
    assert not names["ranges_ordered"].ok


def test_feasibility_missing_source_warns_but_does_not_gate():
    """Label: property
    Parity: vessel_standards:validate_catalogue/with_missing_source
    """
    bad = dict(VESSEL_STANDARDS)
    bad["UNSOURCED"] = VesselStandard(
        standard_id="UNSOURCED", family="CN", class_name="X", formation="motor_vessel",
        loa_m=(90.0, 90.0), beam_m=14.0, draught_m=(2.5, 3.0), dwt_t=(1000.0, 2000.0),
    )
    c = {x.name: x for x in validate_catalogue(bad)}["source_cited"]
    assert not c.ok
    assert c.severity == "warning"
