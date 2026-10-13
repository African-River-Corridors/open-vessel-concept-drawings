"""The README's worked example runs."""
import runpy
from pathlib import Path


def test_quickstart_runs(capsys):
    """Label: snapshot
    Parity: barge_capacity:gas/quickstart_CN_II_motor_90x14.8x6.5_LPG_propane,
        gas_arrangement:fit/quickstart_CN_II_motor_90x14.8_LPG_propane,
        vessel_standards:get_standard/CN_II_motor
    """
    runpy.run_path(str(Path(__file__).resolve().parents[1] / "examples" / "quickstart.py"),
                   run_name="__main__")
    out = capsys.readouterr().out
    assert "Searched   : 1 tank(s) abreast" in out and "2,058 t" in out
