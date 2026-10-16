"""The README's worked example runs."""
import runpy
from pathlib import Path


def test_quickstart_runs(capsys):
    """Label: snapshot
    """
    runpy.run_path(str(Path(__file__).resolve().parents[1] / "examples" / "quickstart.py"),
                   run_name="__main__")
    out = capsys.readouterr().out
    # Re-pinned (issue #237: the per-tank d(Vc) of IGC 2.4.1.1, dome in Vc); was 2,058 t. Re-pinned
    # again (issue #264: the search keeps only arrangements that pass stability); was 2,943 t.
    assert "Searched   : 1 tank(s) abreast" in out and "2,833 t" in out
