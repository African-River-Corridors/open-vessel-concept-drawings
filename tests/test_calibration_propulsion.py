"""The propulsion calibration cases (issue #351, spec 0007): tugs and pushers with a published bollard
pull, run through the Propulsor model only, banded in `bands.yaml`.

`calibration/propulsion_cases.yaml` follows the conventions of the other case files: each case names
the file it was read from and its SHA-256, keeps the source text as published, and a band miss is a
warning, never a failure. The in-sample cases of a type fit its bollard pull factor; the Defaults
Profile takes the fitted values.
"""

import copy
import re

from vessel_designer import calibration
from vessel_designer.calibration import (BANDS, CAL, _assess_installed_power, _assess_propulsion,
                                         _bollard_pull, _fit_bollard_pull_factor,
                                         _fit_efficiency_factor, _installed_power,
                                         _load_propulsion_cases, _propulsion_warnings_for, report)

TYPES = {"azimuth_open", "azimuth_ducted", "shaft_open", "shaft_nozzle", "voith_schneider", "pump_jet"}


def _cases():
    return _load_propulsion_cases()["propulsion_case"]


def _by_id(checks):
    return {b.case: b for b in checks}


def test_every_case_names_its_source_file_hash_and_how_it_is_run():
    """Label: property
    """
    for c in _cases() + _load_propulsion_cases()["not_yet_runnable_propulsion_cases"]:
        assert c["source_file"].startswith("vessels/"), c["id"]
        assert re.fullmatch(r"[0-9a-f]{64}", c["sha256"]), c["id"]
        assert c["published"].strip(), c["id"]
    for c in _cases():
        assert c["run_as_type"] in TYPES, c["id"]
        assert c["diameter_basis"] in {"propeller", "nozzle"}, c["id"]
        assert c["power_basis"] in {"engine", "motor"} and c["drive"] in {"shaft", "azimuth"}, c["id"]
        assert c["confidence"] in {"high", "medium", "low"}, c["id"]
        assert c["url"].startswith("https://") and str(c["retrieved"]) == "2026-10-20", c["id"]
        assert set(c["observed"]) == {"bollard_pull_t"}, c["id"]
        if "held_out" in c:
            assert c["held_out"].strip(), c["id"]


def test_the_fit_ignores_held_out_cases():
    """Label: property
    """
    cases = _cases()
    fitted = _fit_bollard_pull_factor(cases)
    in_sample = [c for c in cases if "held_out" not in c]
    assert _fit_bollard_pull_factor(in_sample) == fitted
    for t, f in fitted.items():
        ids = {e["case"] for e in f.get("evidence", [])}
        assert ids <= {c["id"] for c in in_sample}, t
    # The filter is live: a moved held-out figure leaves the fit alone, a moved in-sample one moves it.
    moved = copy.deepcopy(cases)
    next(c for c in moved if c["id"] == "asd-tug-2312")["observed"]["bollard_pull_t"] *= 2
    assert _fit_bollard_pull_factor(moved) == fitted
    next(c for c in moved if c["id"] == "asd-tug-3212")["observed"]["bollard_pull_t"] *= 2
    assert _fit_bollard_pull_factor(moved)["azimuth_ducted"] != fitted["azimuth_ducted"]


def test_the_committed_factors_hold_the_fit():
    """The Defaults Profile takes the same values: `crates/ovcd-format/tests/propulsor_inputs.rs`.

    Label: property
    """
    import yaml
    fitted = _fit_bollard_pull_factor(_cases())
    committed = yaml.safe_load((CAL / "factors.yaml").read_text(encoding="utf-8"))["bollard_pull_factor"]
    assert committed == fitted


def test_every_case_is_inside_its_band_or_its_miss_is_accepted():
    """Label: sourced - docs/research/stern-and-tugs.md sec. 3.5 and sec. 10 (the vessels), spec 0007 (the bands); each figure as its source prints it, read from the files named in propulsion_cases.yaml.
    """
    accepted = {c["id"] for c in _cases() if "bollard_pull_t" in c.get("accepted_miss", {})}
    for b in _assess_propulsion():
        assert b.band_pct == BANDS["bands"]["bollard_pull_t"]["pct"][b.confidence]
        if b.case in accepted:
            assert not b.inside, f"{b.case}: an accepted miss must be outside its band"
        else:
            assert b.inside, (b.case, b.err_pct)


def test_the_bigger_propeller_of_the_same_power_pulls_more_and_both_are_within_10_percent():
    """Label: sourced - the ASD 2312 and ASD 2811 product sheets (same power, 2.8 and 2.4 m, 70 and 60 t), docs/research/stern-and-tugs.md sec. 3.5.
    """
    checks = _by_id(_assess_propulsion())
    big, small = checks["asd-tug-2312"], checks["asd-tug-2811"]
    assert big.model > small.model
    assert abs(big.err_pct) < 10.0 and abs(small.err_pct) < 10.0
    assert big.held_out and small.held_out


def test_a_moved_figure_fires_a_warning_and_an_accepted_miss_gives_its_reason():
    """Label: property
    """
    lines = _propulsion_warnings_for(_assess_propulsion())
    assert len(lines) == 1 and "cds-4115" in lines[0] and "accepted:" in lines[0]
    cases = copy.deepcopy(_load_propulsion_cases())
    next(c for c in cases["propulsion_case"] if c["id"] == "demi-k")["observed"]["bollard_pull_t"] = 40.0
    lines = _propulsion_warnings_for(_assess_propulsion(cases))
    assert any(line.startswith("::warning") and "demi-k" in line and "held-out" in line for line in lines)


def test_the_model_scales_with_kappa_b_and_the_report_lists_every_case():
    """Label: property
    """
    c = _cases()[0]
    assert abs(_bollard_pull(c, 2.0) - 2.0 * _bollard_pull(c, 1.0)) < 1e-9
    text = report()
    assert "## 6. Bollard pull" in text
    for c in _cases() + _load_propulsion_cases()["not_yet_runnable_propulsion_cases"]:
        assert f"| {c['id']} |" in text, c["id"]
    assert calibration.main_check() == 0


def _power_cases():
    return _load_propulsion_cases()["installed_power_case"]


def test_the_efficiency_fit_uses_the_in_sample_ships_and_is_committed():
    """Label: property
    """
    import yaml
    cases = _power_cases()
    fitted = _fit_efficiency_factor(cases)
    assert _fit_efficiency_factor([c for c in cases if "held_out" not in c]) == fitted
    committed = yaml.safe_load((CAL / "factors.yaml").read_text(encoding="utf-8"))
    assert committed["propulsor_efficiency_factor"] == fitted
    # The Installed Power falls as kappa rises: the fit has a direction.
    c = cases[0]
    assert _installed_power(c, 0.9) < _installed_power(c, 0.45)


def test_the_installed_power_cases_are_inside_their_bands():
    """Label: sourced - the CDS 110 and CDS 135 Tanker brochures (2x 500 and 2x 709 kW at a design speed of 18 km/h), read from the files named in propulsion_cases.yaml; the bands of spec 0007.
    """
    checks = _assess_installed_power()
    assert {b.case for b in checks} == {c["id"] for c in _power_cases()}
    for b in checks:
        assert b.inside, (b.case, b.err_pct)
        assert b.band_pct == BANDS["bands"]["installed_power_kw"]["pct"][b.confidence]
    assert any(b.held_out for b in checks)
    assert "## 7. Installed power" in report()
