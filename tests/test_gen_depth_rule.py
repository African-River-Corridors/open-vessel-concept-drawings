"""The depth-rule table `catalogue/depth_rule.md` is generated, and says what the data says (issue #120).

`tools/gen_depth_rule.py` writes it from the built-in Defaults Profile and the Reference Vessels. Risk
stopped: the table is edited by hand, or the profile's ratios or the catalogue change and the table is
left behind, so a reader sees a rule value the engine does not give.

Label: property
"""
import importlib.util
import json
import re
import subprocess
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location("gen_depth_rule", ROOT / "tools" / "gen_depth_rule.py")
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)


def test_the_committed_table_is_what_the_generator_writes():
    """Label: property"""
    assert gen.OUT.read_text(encoding="utf-8") == gen.render()
    ok = subprocess.run([sys.executable, str(ROOT / "tools" / "gen_depth_rule.py"), "--check"])
    assert ok.returncode == 0


def test_every_reference_vessel_with_a_published_depth_is_listed():
    """Label: property"""
    vessels = yaml.safe_load((ROOT / "catalogue" / "reference_vessels.yaml").read_text(encoding="utf-8"))["vessels"]
    with_depth = [v for v in vessels if v.get("depth_m") is not None]
    assert len(with_depth) == 41
    text = gen.OUT.read_text(encoding="utf-8")
    for v in with_depth:
        assert f"| {v['name'].replace('|', '/')} |" in text, v["name"]
    assert "41 Reference Vessels have a published depth" in text


def test_the_ratios_in_the_table_are_the_profiles_and_the_rule_column_is_ratio_times_beam():
    """Label: property"""
    profile = json.loads(gen.PROFILE.read_text(encoding="utf-8"))["inputs"]
    ratios = profile["depth_to_beam_ratio"]["default_by_cargo_group"]
    assert ratios == {"dry_bulk": 0.33, "liquid": 0.30, "gas": 0.55}
    data, got, slender = gen.rows()
    assert got == ratios and slender == 25.0
    for r in data:
        if r["group"] and r["beam"]:
            assert r["rule"] == round(ratios[r["group"]] * r["beam"], 3)
        else:
            assert r["rule"] is None
    # The gas range of the vessels is the one the maintainer gave: 0.44 to 0.66.
    gas = [r["ratio"] for r in data if r["group"] == "gas" and r["ratio"]]
    assert round(min(gas), 2) == 0.44 and round(max(gas), 2) == 0.66
    assert any(r["group"] is None for r in data)          # a tug or pusher with no cargo: no rule


def test_the_profile_says_what_the_rule_is_and_the_old_basis_text_is_gone():
    """Label: property"""
    text = gen.PROFILE.read_text(encoding="utf-8")
    assert "the catalogue holds no depths" not in text
    profile = json.loads(text)["inputs"]
    assert "keel to main deck at the side" in profile["moulded_depth_m"]["basis"]
    assert "catalogue/depth_rule.md" in profile["depth_to_beam_ratio"]["basis"]
    assert profile["length_to_depth_ratio"]["soft"]["max"] == 25.0
