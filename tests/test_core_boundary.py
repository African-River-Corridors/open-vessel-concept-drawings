"""The calculation core stays pure: it imports only itself, the standard library."""
import ast
import sys
from pathlib import Path

CORE = Path(__file__).resolve().parents[1] / "src" / "vessel_designer" / "core"
ALLOWED_THIRD_PARTY: set[str] = set()


def test_core_imports_nothing_outside_itself():
    bad = []
    files = sorted(CORE.glob("*.py"))
    # Risk stopped: a scan of zero files passes. This test reads the source tree, so it runs
    # in `test`, not in the wheel job (which has no src/).
    assert files, f"no core modules found under {CORE}"
    for path in files:
        for node in ast.walk(ast.parse(path.read_text())):
            if isinstance(node, ast.Import):
                names = [a.name for a in node.names]
            elif isinstance(node, ast.ImportFrom) and node.level == 0:
                names = [node.module]
            else:
                continue
            for name in names:
                top = name.split(".")[0]
                if top not in sys.stdlib_module_names and top not in ALLOWED_THIRD_PARTY \
                        and not name.startswith("vessel_designer.core"):
                    bad.append(f"{path.name}: {name}")
    assert not bad, bad
