"""The parity driver writes Reference Results in the spec 0001 text form.

Floats are shortest round-trip text; NaN and infinity are tagged strings; a ValueError is an
`error` result. Risk stopped: the oracle file loses precision or cannot carry non-finite values.
"""
import importlib.util
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
_spec = importlib.util.spec_from_file_location("parity_driver", ROOT / "parity" / "driver.py")
driver = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(driver)


def test_non_finite_floats_become_tagged_strings():
    out = json.loads(driver.dumps(driver.to_json([math.nan, math.inf, -math.inf, 1.5])))
    assert out == ["float:nan", "float:inf", "float:-inf", 1.5]


def test_floats_are_written_as_shortest_round_trip_text():
    text = driver.dumps({"x": 0.1 + 0.2, "y": 1e-9, "z": 450.0})
    assert '"x": 0.30000000000000004' in text
    assert '"y": 1e-09' in text
    assert '"z": 450.0' in text
    assert json.loads(text)["x"] == 0.1 + 0.2


def test_value_error_is_recorded_with_its_message():
    out = driver.run_case("cargo", {"fn": "get_cargo", "args": {"cargo_type": "coal"}})
    assert out["error"]["type"] == "ValueError"
    assert out["error"]["message"].startswith("Unknown cargo type 'coal'. Valid types: LNG, ")


def test_driver_runs_the_frozen_copy_not_the_shipped_package():
    mod = driver._module("core.cargo")
    assert mod.__name__ == "vessel_designer_frozen.core.cargo"
    assert "frozen_python" in Path(mod.__file__).parts


def test_key_error_is_recorded_with_its_class_name():
    out = driver.run_case("vessel_standards", {"fn": "get_standard", "args": {"standard_id": "nope"}})
    assert out["error"]["type"] == "KeyError"
    assert "unknown vessel standard" in out["error"]["message"]


def test_nested_inputs_are_built_from_plain_json():
    mod = driver._module("core.resistance")
    built = driver.build(mod.ResistanceInputs, {"tug": {"engine_power_kw": 5.0},
                                                "environments": [{"name": "x"}]})
    assert isinstance(built.tug, mod.TugInputs) and built.tug.engine_power_kw == 5.0
    assert isinstance(built.environments[0], mod.EnvironmentInputs)
