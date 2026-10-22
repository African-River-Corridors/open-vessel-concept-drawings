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
    # again (issue #264: the search keeps only arrangements that pass stability); was 2,943 t. Re-pinned
    # again (issue #277: the wheelhouse sightline blocks the search); was 2,833 t. Re-pinned again
    # (issue #410: the covering above D is in the lightship; one tank abreast fails the
    # light-condition angle of max GZ); was 1 tank abreast and 2,021 t.
    # Re-pinned again (issue #498: the angle of max GZ alone in the light condition is a warning, so
    # one tank abreast is feasible again); was 2 tanks abreast and 1,406 t.
    assert "Searched   : 1 tank(s) abreast" in out and "2,025 t" in out   # was 2,021 t (issue #280)
