"""The clipped hull STL of `ovd export --format stl --below <condition>` is a closed mesh that a
BEM code reads, and holds the displaced volume of the condition (issue #461, spec 0008 "Phase 2:
exports", "Capytaine").

Skipped when Capytaine (with meshio and trimesh, which it reads and checks STL with) or the `ovd`
binary is not there: pip install capytaine meshio trimesh; cargo build -p ovd-cli, or OVD_BIN.

Label: property"""
import json
import os
import shutil
import subprocess
from pathlib import Path

import pytest

cpt = pytest.importorskip("capytaine")
pytest.importorskip("meshio")
pytest.importorskip("trimesh")

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "crates" / "ovd-format" / "tests" / "fixtures"


def _ovd() -> str:
    candidates = [os.environ.get("OVD_BIN")]
    for target in (os.environ.get("CARGO_TARGET_DIR"), str(ROOT / "target")):
        if target:
            candidates += [f"{target}/debug/ovd", f"{target}/release/ovd"]
    candidates.append(shutil.which("ovd"))
    found = [c for c in candidates if c and Path(c).is_file()]
    if found:
        # The newest build: a debug and a release binary may both be there.
        return max(found, key=lambda c: Path(c).stat().st_mtime)
    pytest.skip("no ovd binary (cargo build -p ovd-cli, or set OVD_BIN)")


def _conditions(binary: str, fixture: Path):
    """(name, displaced volume m3) of each stability condition of the fixture's first Option."""
    design = json.loads(fixture.read_text())
    density = design["options"][0]["inputs"].get("water_density_t_per_m3", {}).get("value") or 1.0
    run = subprocess.run([binary, "run", "--json", str(fixture)], capture_output=True, text=True)
    option = json.loads(run.stdout)["options"][0]
    conditions = (option.get("results") or {}).get("stability", {}).get("conditions", [])
    return [(c["name"], c["displacement_t"] / density) for c in conditions]


@pytest.mark.parametrize("fixture", ["gas_igc_2g_exact", "gas_igc_2g_self_propelled",
                                     "crude_integral_tanks_pushed"])
def test_the_clipped_hull_is_closed_and_holds_the_displaced_volume(fixture, tmp_path):
    """Label: property"""
    binary = _ovd()
    path = FIXTURES / f"{fixture}.vdesign"
    conditions = _conditions(binary, path)
    assert conditions, "the fixture has stability conditions"
    for name, volume in conditions:
        out = tmp_path / "hull.stl"
        done = subprocess.run([binary, "export", "--format", "stl", "--below", name, "-o", str(out),
                               str(path)], capture_output=True, text=True)
        assert done.returncode == 0, done.stderr
        mesh = cpt.load_mesh(str(out), file_format="stl")
        # Closed and consistently wound, as Capytaine's trimesh export finds it.
        t = mesh.export_to_trimesh()
        assert t.is_watertight and t.is_winding_consistent, name
        # The displaced volume within 1 % of OVD's.
        assert mesh.volume == pytest.approx(volume, rel=0.01), name
