"""Packaging contract (ticket 26): maturin build, ovcd-py skeleton, release gate, no publish."""
import re
import tomllib
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
RELEASE = ROOT / ".github" / "workflows" / "release.yml"
CI = ROOT / ".github" / "workflows" / "ci.yml"


def pyproject() -> dict:
    return tomllib.loads((ROOT / "pyproject.toml").read_text(encoding="utf-8"))


def workflow(path: Path) -> dict:
    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    data["on"] = data.pop(True, data.get("on"))  # YAML reads a bare `on` as True
    return data


# ---------- project file ----------

def test_maturin_is_the_build_backend():
    build = pyproject()["build-system"]
    assert build["build-backend"] == "maturin"
    assert any(r.startswith("maturin") for r in build["requires"])
    assert not any("hatch" in r for r in build["requires"])


def test_distribution_name_is_vessel_designer():
    assert pyproject()["project"]["name"] == "vessel-designer"


def test_maturin_builds_ovcd_py_as_a_submodule_of_the_package():
    m = pyproject()["tool"]["maturin"]
    assert m["manifest-path"] == "crates/ovcd-py/Cargo.toml"
    assert m["module-name"] == "vessel_designer._native"
    assert m["python-source"] == "src"
    assert "pyo3/extension-module" in m["features"]


def test_numpy_is_not_a_dependency():
    # The shipped package never needs numpy. Only the frozen Python oracle does, so it may sit in the
    # `parity` extra (which CI installs for `cargo xtask regen` and the oracle guard) and nowhere else.
    proj = pyproject()["project"]
    extras = proj.get("optional-dependencies", {})
    runtime = proj["dependencies"] + [d for k, ds in extras.items() if k != "parity" for d in ds]
    assert not [d for d in runtime if re.match(r"numpy\b", d, re.I)]
    assert [d for d in extras.get("parity", []) if re.match(r"numpy\b", d, re.I)]


def test_no_package_module_imports_numpy():
    files = list((ROOT / "src").rglob("*.py"))
    assert files, "no package modules found: scanning nothing"
    hits = [p.name for p in files if re.search(r"^\s*(import|from)\s+numpy", p.read_text(encoding="utf-8"), re.M)]
    assert not hits, hits


def test_calculator_is_gone_from_the_package():
    assert not (ROOT / "src/vessel_designer/core/calculator.py").exists()
    files = list((ROOT / "src").rglob("*.py"))
    assert files, "no package modules found: scanning nothing"
    for p in files:
        assert not re.search(r"core\.calculator|import calculator|from \.calculator", p.read_text(encoding="utf-8")), p


def test_example_rates_ship_in_the_package_and_match_the_reader_copy():
    shipped = ROOT / "src/vessel_designer/data/barge_cost_rates.example.yaml"
    assert shipped.read_bytes() == (ROOT / "examples/barge_cost_rates.example.yaml").read_bytes()


# ---------- ovcd-py skeleton ----------

def test_core_crates_never_depend_on_pyo3():
    for crate in ("ovcd-core", "ovcd-format", "ovcd-cli"):
        assert "pyo3" not in (ROOT / "crates" / crate / "Cargo.toml").read_text(encoding="utf-8"), crate


def test_ovcd_py_is_abi3_for_python_312():
    text = (ROOT / "crates/ovcd-py/Cargo.toml").read_text(encoding="utf-8")
    assert "abi3-py312" in text
    assert 'crate-type = ["cdylib"]' in text


# ---------- release gate ----------

def test_release_runs_per_release_and_by_hand_but_not_nightly():
    on = workflow(RELEASE)["on"]
    assert "release" in on and "workflow_dispatch" in on
    assert "schedule" not in on


def test_ci_can_be_called_by_the_release_workflow():
    assert "workflow_call" in workflow(CI)["on"]
    jobs = workflow(CI)["jobs"]
    assert "test" in jobs and "release-guard" in jobs


def test_every_build_job_waits_for_test_and_release_guard():
    jobs = workflow(RELEASE)["jobs"]
    gate = [n for n, j in jobs.items() if j.get("uses", "").endswith("ci.yml")]
    assert gate, "release.yml must call ci.yml (jobs `test` and `release-guard`)"
    builders = [n for n, j in jobs.items() if n not in gate]
    assert builders
    for name in builders:
        needs = jobs[name].get("needs", [])
        needs = [needs] if isinstance(needs, str) else needs
        assert set(gate) <= set(needs), f"{name} does not wait for {gate}"
        assert "always()" not in str(jobs[name].get("if", "")), f"{name} can run past a failed gate"


def test_wheels_build_on_all_four_targets():
    jobs = workflow(RELEASE)["jobs"]
    wheels = next(j for n, j in jobs.items() if "wheel" in n)
    assert set(wheels["strategy"]["matrix"]["os"]) >= {
        "ubuntu-latest", "macos-15-intel", "macos-latest", "windows-latest"}
    text = RELEASE.read_text(encoding="utf-8")
    assert "--only-binary" in text, "the wheel must install with no compiler"
    assert "pytest" in text and "pip check" in text


def test_binaries_build_for_windows_and_macos():
    jobs = workflow(RELEASE)["jobs"]
    bins = next(j for n, j in jobs.items() if "binar" in n)
    assert set(bins["strategy"]["matrix"]["os"]) >= {"macos-15-intel", "macos-latest", "windows-latest"}
    assert "ovcd-cli" in RELEASE.read_text(encoding="utf-8")


def test_nothing_publishes_to_pypi():
    for path in (RELEASE, CI):
        text = path.read_text(encoding="utf-8").lower()
        for banned in ("pypi-publish", "twine", "maturin publish", "id-token", "environment: pypi"):
            assert banned not in text, f"{path.name}: {banned}"


def test_wheel_job_skips_the_tests_that_scan_files_it_does_not_copy():
    """The wheel job copies tests/, examples/, calibration/ and catalogue/ only. A test that
    scans src/ or parity/ would scan zero files there, so the job must skip it."""
    text = (ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8")
    for name in ("test_parity_through_python.py", "test_test_audit.py", "test_pystr_tables.py",
                 "test_vdesign_edge_sweep.py"):
        assert f"--ignore=tests/{name}" in text, name


def test_version_is_0_2_0_everywhere():
    """Ticket 27: one version for the package and every `ovcd-*` crate."""
    assert pyproject()["project"]["version"] == "0.2.0"
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    assert cargo["workspace"]["package"]["version"] == "0.2.0"
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    ours = {p["name"]: p["version"] for p in lock["package"]
            if p["name"] in ("ovcd-core", "ovcd-format", "ovcd-cli", "ovcd-py", "xtask")}
    assert ours == dict.fromkeys(("ovcd-core", "ovcd-format", "ovcd-cli", "ovcd-py", "xtask"), "0.2.0")
    for crate in ("ovcd-core", "ovcd-format", "ovcd-cli", "ovcd-py"):
        manifest = tomllib.loads((ROOT / "crates" / crate / "Cargo.toml").read_text(encoding="utf-8"))
        assert manifest["package"]["version"] == {"workspace": True}, crate
