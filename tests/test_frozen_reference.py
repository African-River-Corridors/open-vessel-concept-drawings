"""The frozen parity copy of the Python code can never be imported by the shipped package's name.

Spec 0001: the pure-Python code that writes the Reference Results lives under a different import
name, so a test can never import the wrong one by accident. Risk stopped: the parity oracle and the
shipped package silently become the same code (or the shipped name loads the frozen copy).
"""
import ast
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FROZEN_ROOT = ROOT / "parity" / "frozen_python"
FROZEN_PKG = "vessel_designer_frozen"
SHIPPED_PKG = "vessel_designer"


def _imported_names(path: Path) -> list[str]:
    names = []
    for node in ast.walk(ast.parse(path.read_text(encoding="utf-8"))):
        if isinstance(node, ast.Import):
            names += [a.name for a in node.names]
        elif isinstance(node, ast.ImportFrom) and node.level == 0 and node.module:
            names.append(node.module)
        elif isinstance(node, ast.Call) and getattr(node.func, "attr", "") == "files":
            names += [a.value for a in node.args if isinstance(a, ast.Constant)]
    return names


def test_frozen_root_holds_only_the_frozen_package():
    entries = sorted(p.name for p in FROZEN_ROOT.iterdir() if p.name != "__pycache__")
    assert entries == [FROZEN_PKG]


def test_frozen_code_never_names_the_shipped_package():
    bad = []
    for path in (FROZEN_ROOT / FROZEN_PKG).rglob("*.py"):
        for name in _imported_names(path):
            if name.split(".")[0] == SHIPPED_PKG:
                bad.append(f"{path.relative_to(ROOT)}: {name}")
    assert not bad, bad


def test_shipped_name_resolves_to_the_shipped_package_even_with_frozen_root_on_path():
    code = (
        "import sys; sys.path.insert(0, sys.argv[1]);"
        "import vessel_designer, vessel_designer.core.cargo as c;"
        "print(c.__file__)"
    )
    out = subprocess.run([sys.executable, "-c", code, str(FROZEN_ROOT)],
                         capture_output=True, text=True, check=True).stdout.strip()
    assert "frozen" not in out
    assert Path(out).parts[-3:] == (SHIPPED_PKG, "core", "cargo.py")


def test_frozen_package_imports_under_its_own_name():
    code = (
        "import sys; sys.path.insert(0, sys.argv[1]);"
        "from vessel_designer_frozen.core.cargo import get_cargo;"
        "print(get_cargo('LNG').density)"
    )
    out = subprocess.run([sys.executable, "-c", code, str(FROZEN_ROOT)],
                         capture_output=True, text=True, check=True).stdout.strip()
    assert out == "450.0"
