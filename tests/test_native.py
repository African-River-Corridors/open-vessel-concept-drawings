"""The compiled core (ticket 26 skeleton, ticket 27 switch). These tests also run against the installed wheel in the release gate."""
import importlib.metadata
import subprocess
import sys


def test_native_module_is_built_and_reports_the_package_version():
    from vessel_designer import _native

    assert _native.version() == importlib.metadata.version("vessel-designer")


def test_package_calculates_in_the_native_module():
    """Importing the package and calling a calculation loads the compiled core (ticket 27)."""
    code = ("import sys, vessel_designer.simulate as s; s.cargo_options(); "
            "sys.exit(0 if 'vessel_designer._native' in sys.modules else 1)")
    assert subprocess.run([sys.executable, "-c", code]).returncode == 0


def test_example_rates_ship_inside_the_package():
    from importlib import resources

    assert resources.files("vessel_designer").joinpath("data", "barge_cost_rates.example.yaml").is_file()
