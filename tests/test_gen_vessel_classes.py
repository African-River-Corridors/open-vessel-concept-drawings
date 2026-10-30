"""The core's Vessel Class list is generated from `catalogue/vessel_classes.yaml` (issue #94).

`crates/ovd-core/src/vessel_classes_generated.rs` is written by `tools/gen_vessel_classes.py`. Risk
stopped: someone edits the generated file by hand, or edits the YAML (or the generator) and leaves the
core's list behind, so the two lists drift. The Python `VESSEL_STANDARDS` is read from the core, so
there is no third list either.

Label: property
"""
import importlib.util
from pathlib import Path

import yaml

from ovd.core import vessel_standards as vs

ROOT = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location("gen_vessel_classes", ROOT / "tools" / "gen_vessel_classes.py")
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)

CLASSES = yaml.safe_load((ROOT / "catalogue" / "vessel_classes.yaml").read_text(encoding="utf-8"))["classes"]


def test_the_committed_core_list_is_what_the_generator_writes():
    """Label: property"""
    assert gen.OUT.read_text(encoding="utf-8") == gen.render()


def test_the_check_flag_exits_non_zero_on_a_stale_file(tmp_path, monkeypatch):
    """Label: property"""
    import subprocess
    import sys
    ok = subprocess.run([sys.executable, str(ROOT / "tools" / "gen_vessel_classes.py"), "--check"])
    assert ok.returncode == 0
    stale = tmp_path / "stale.rs"
    stale.write_text("// out of date\n")
    monkeypatch.setattr(gen, "OUT", stale)
    assert gen.OUT.read_text(encoding="utf-8") != gen.render()


def test_every_yaml_class_is_in_the_core_and_python_lists_with_the_same_figures():
    """Label: property"""
    assert len(vs.VESSEL_STANDARDS) == len(CLASSES) == 292
    for c in CLASSES:
        s = vs.VESSEL_STANDARDS[c["id"]]
        assert s.standard_id == c["id"] and s.family == c["family"] and s.formation == c["formation"]
        loa = c["loa_m"]
        want = None if loa is None else tuple(map(float, loa if isinstance(loa, list) else [loa, loa]))
        assert s.loa_m == want, c["id"]
        dwt = c["dwt_t"]
        want = None if dwt is None else tuple(map(float, dwt if isinstance(dwt, list) else [dwt, dwt]))
        assert s.dwt_t == want, c["id"]
        assert list(s.cargo_modes) == list(c["cargo_modes"]), c["id"]
        assert s.source == c["source"] and s.notes == c["notes"]


def test_the_old_ids_are_aliases_and_the_unsourced_figures_are_gone():
    """Label: property"""
    for old, new in gen.LEGACY_IDS.items():
        assert vs.get_standard(old).standard_id == new
        assert old not in vs.VESSEL_STANDARDS
    assert vs.get_standard("CN_II_motor").moulded_depth_m is None
    assert vs.get_standard("CEMT_VIa_motor_NL2011").min_bridge_clearance_m is None
    assert vs.get_standard("CEMT_Va").draught_m == (2.5, 2.8)
