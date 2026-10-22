#!/usr/bin/env python3
"""Design-impact check: do designs move on this branch? (spec 0001, ticket 28)

A fixed set of Designs (the calibration vessels plus a small sweep) is run through `ovcd` twice,
once built from the base (the merge base with `origin/main`, or the parent commit when this is
`main` itself) and once built from this tree: `ovcd fill`, then `ovcd run --json`. Nothing is
stored, so nothing goes stale. If every result is identical the report says "no design change".
If anything moved, the report holds a before/after table.

    python3 tools/design_impact.py report [--publish --pr N]   report only, never fails on a change
    python3 tools/design_impact.py gate                        report, and fail when a change is
                                                               missing from CHANGELOG.md
    python3 tools/design_impact.py changelog [--key NAME]      write the table into CHANGELOG.md
    python3 tools/design_impact.py publish-artifact DIR --head-sha SHA --head-repo OWNER/NAME
                                                               publish a report built by a pull request
                                                               workflow (used for fork pull requests)

The report goes to target/design-impact/report.md (and summary.json), is printed, and is added to
the GitHub step summary when there is one. `--publish` puts it into the pull request body between
two markers and adds or removes the `design-change` label (needs GH_TOKEN and the `gh` CLI).

A design change needs no sign-off. The changelog entry exists so it can be diagnosed later.
Standard library plus PyYAML (a dependency of the package).
"""
from __future__ import annotations

import argparse
import datetime
import importlib.util
import json
import math
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "target" / "design-impact"
CHANGELOG = ROOT / "CHANGELOG.md"
START, END = "<!-- design-impact:start -->", "<!-- design-impact:end -->"
# Top-level results keys that are not results (drawing geometry, what a view marks). The results hash
# leaves them out too: UNHASHED_KEYS in crates/ovcd-format/src/run.rs. tests/test_design_impact_unhashed_keys.py
# checks that the two lists agree.
UNHASHED_KEYS = ("drawing", "stability_marks", "cost_masses", "cost", "text_ids", "rule_trace", "calibration")

LABEL = "design-change"
MAX_ROWS = 40
CHANGELOG_HEAD = (
    "# Changelog\n\n"
    "## Design changes\n\n"
    "Each entry is a before/after table written by `python3 tools/design_impact.py changelog` when a\n"
    "branch moved the results of the fixed design set. No sign-off is needed; the entries are kept so\n"
    "a change can be traced later. Newest first.\n"
)
SECTION = "## Design changes"
ENTRIES_AT = "\n### "


# --- the fixed case set -------------------------------------------------------------------------

def _inputs(cargo, containment, formation, loa, beam, depth, **more):
    d = {"cargo": cargo, "hull_form": "raked_box", "formation": {"kind": formation},
         "length_overall_m": loa, "beam_m": beam, "moulded_depth_m": depth,
         "cargo_containment": containment}
    if depth is None:               # left out: the Defaults Profile's rule gives it
        del d["moulded_depth_m"]
    d.update(more)
    return d


GAS_CARGOES = ("LNG", "LNG_ethane", "LPG_propane", "LPG_butane", "LPG_mix")
# The CN note gives no depth. Draught plus this many metres is a stand-in that keeps the set stable.
CN_DEPTH_ABOVE_DRAUGHT_M = 1.5
DEPTH_NOTE = (f"The CN-derived cases take their length, beam, draught and speed from a published draft note. "
              f"The note gives no moulded depth, so each uses the draught plus {CN_DEPTH_ABOVE_DRAUGHT_M:g} m "
              f"as a stand-in. They are sweep points, not calibration comparisons.")

# One case for each optional input, on a hull where the input moves a result. Each entry:
# (case name, cargo, containment, formation, loa, beam, depth, extra inputs).
_CHANNEL = {"channel_depth_m": 5.0, "channel_bottom_width_m": 60.0}
_DEEP_CHANNEL = {"channel_depth_m": 3.0, "channel_bottom_width_m": 30.0}      # shallow and narrow
OPTIONAL_INPUT_CASES = (
    ("block_coefficient", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"block_coefficient": 0.7}),
    ("min_freeboard_m", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5, {"min_freeboard_m": 0.9}),
    ("lightship_factor_t_per_m3", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {"lightship_factor_t_per_m3": 0.2}),
    ("propulsive_efficiency", "bauxite", "box_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"propulsive_efficiency": 0.55, "max_speed_kn": 9.0}),
    ("air_draft_limit_m", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5, {"air_draft_limit_m": 6.0}),
    ("channel_current_kn", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {**_DEEP_CHANNEL, "channel_current_kn": 2.0}),
    ("channel_bank_slope", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {**_DEEP_CHANNEL, "channel_bank_slope": 2.0}),
    ("channel_under_keel_clearance_m", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {**_DEEP_CHANNEL, "channel_under_keel_clearance_m": 0.8}),
    ("fill_ratio", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"fill_ratio": 0.85}),
    ("max_tank_od_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 20.0, 6.0,
     {"max_tank_od_m": 12.0}),
    # The dry-bulk hold inputs (issue #4, slice 1). Each is set to its default, which is the value the engine
    # used before it was an input. A base from before the input does not know it; `run_case` leaves
    # an input the base refuses out, so these show that the head, with the value typed, gives the
    # base's results.
    ("hold_wing_void_width_m", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {"hold_wing_void_width_m": 1.0}),
    ("hold_double_bottom_m", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {"hold_double_bottom_m": 1.2}),
    ("hatch_coaming_height_m", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {"hatch_coaming_height_m": 1.0}),
    ("hold_fill_ratio", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {"hold_fill_ratio": 0.95}),
    ("hopper_slope_h_per_v", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"hopper_slope_h_per_v": 1.0}),
    ("n_holds", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5, {"n_holds": 3}),
    ("inner_bottom_plate_mm", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"inner_bottom_plate_mm": 14.0}),
    ("hopper_plate_mm", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"hopper_plate_mm": 14.0}),
    ("hold_bulkhead_plate_mm", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"hold_bulkhead_plate_mm": 11.0}),
    ("hatch_coaming_plate_mm", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"hatch_coaming_plate_mm": 10.0}),
    # The hull end shapes (issue #4, slice 2), set to their defaults in the same way.
    ("bow_rake_length_to_depth_ratio", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"bow_rake_length_to_depth_ratio": 1.6}),
    ("pushed_stern_void_m", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {"pushed_stern_void_m": 3.0}),
    ("bulk_end_bulkhead_m", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {"bulk_end_bulkhead_m": 0.5}),
    # The gas tank arrangement inputs (issue #4, slice 3), set to their defaults in the same way.
    ("tanks_in_line", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tanks_in_line": 2}),
    ("tank_support_height_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_support_height_m": 0.5}),
    ("tank_dome_height_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_dome_height_m": 2.0}),
    # The dome diameter (issue #237) is new with the per-tank rule: a design that sets it to its
    # default has the same results as one that leaves it out.
    ("tank_dome_diameter_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_dome_diameter_m": 2.0}),
    ("wing_tank_gap_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"wing_tank_gap_m": 0.6}),
    ("wing_tank_height_ratio", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"wing_tank_height_ratio": 0.66}),
    ("tank_wall_thickness_mm", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_wall_thickness_mm": 25.0}),
    # The weather shield (issue #410, part 1b), set to its defaults: the same results as leaving it out.
    ("weather_shield_thickness_mm", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4,
     6.0, {"weather_shield_thickness_mm": 6.0}),
    ("weather_shield_clearance_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4,
     6.0, {"weather_shield_clearance_m": 1.5}),
    # The aft block inputs (issue #4, slice 4), set to their defaults in the same way. A self-propelled
    # hull carries the block; the unit counts default by Cargo phase (gas 1 battery and 2
    # accommodation units, dry bulk 2 and 1), so each has a gas and a dry case.
    ("thruster_count", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"thruster_count": 3}),
    ("thruster_envelope_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"thruster_envelope_m": 2.6}),
    ("thruster_clearance_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"thruster_clearance_m": 0.6}),
    ("thruster_room_length_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"thruster_room_length_m": 3.5}),
    ("unit_height_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"unit_height_m": 2.896}),
    ("unit_side_gap_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"unit_side_gap_m": 0.9}),
    ("unit_row_gap_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"unit_row_gap_m": 0.8}),
    ("unit_foundation_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"unit_foundation_m": 0.3}),
    ("wheelhouse_length_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"wheelhouse_length_m": 3.6}),
    ("wheelhouse_width_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"wheelhouse_width_m": 6.4}),
    ("wheelhouse_height_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"wheelhouse_height_m": 3.4}),
    ("wheelhouse_gap_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"wheelhouse_gap_m": 0.3}),
    ("wheelhouse_eye_height_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"wheelhouse_eye_height_m": 1.5}),
    ("sightline_margin_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"sightline_margin_m": 0.5}),
    ("wheelhouse_max_lift_m", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"wheelhouse_max_lift_m": 6.0}),
    ("battery_unit_count", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"battery_unit_count": 1}),
    ("battery_unit_count_dry", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"battery_unit_count": 2}),
    ("accommodation_unit_count", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"accommodation_unit_count": 2}),
    ("accommodation_unit_count_dry", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"accommodation_unit_count": 1}),
    ("aft_block_all_dry", "bauxite", "box_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"thruster_count": 3, "thruster_envelope_m": 2.6, "thruster_clearance_m": 0.6, "thruster_room_length_m": 3.5, "unit_height_m": 2.896, "unit_side_gap_m": 0.9, "unit_row_gap_m": 0.8, "unit_foundation_m": 0.3, "wheelhouse_length_m": 3.6, "wheelhouse_width_m": 6.4, "wheelhouse_height_m": 3.4, "wheelhouse_gap_m": 0.3, "wheelhouse_eye_height_m": 1.5, "sightline_margin_m": 0.5, "wheelhouse_max_lift_m": 6.0, "battery_unit_count": 2, "accommodation_unit_count": 1}),
    # The Type C tank design inputs (issue #292), typed away from their defaults on a hull where each
    # moves a result. The defaults themselves depend on the Cargo and the LPG carriage condition, so
    # the sweep below also runs every LPG Cargo in each of the three conditions.
    ("lpg_carriage_condition", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"lpg_carriage_condition": "semi_pressurised"}),
    ("tank_design_pressure_mpa", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_design_pressure_mpa": 1.0}),
    # Issue #315: the loading limit basis and the loading temperature.
    ("lpg_loading_limit_basis", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"lpg_loading_limit_basis": "igc_15_5_2"}),
    ("lpg_loading_temperature_c", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"lpg_loading_temperature_c": 30.0}),
    ("tank_design_temperature_c", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_design_temperature_c": -48.0}),
    ("tank_material", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_material": "304L"}),
    ("tank_allowable_stress_mpa", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_allowable_stress_mpa": 200.0}),
    # Issue #375: the head type, away from the LNG default.
    ("tank_head_type", "LNG", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_head_type": "ellipsoidal_2to1"}),
    # Issue #374: the sea-going accelerations of IGC 4.28.2 in the liquid pressure.
    ("tank_liquid_pressure", "LNG", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_liquid_pressure": "igc_4_28_2"}),
    # Issue #313: the old basis gives S690QL1 its 285 MPa back.
    ("tank_allowable_stress_basis", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_allowable_stress_basis": "published_design"}),
    ("tank_insulation_type", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_insulation_type": "pu_foam"}),
    ("tank_insulation_thickness_mm", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_insulation_thickness_mm": 200.0}),
    ("tank_structure_factor", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"tank_structure_factor": 1.2}),
    # The environment inputs (issue #4, slice 6), set to their defaults in the same way. The squat
    # margin matters only in a channel, so that case has one.
    ("water_density_t_per_m3", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {"water_density_t_per_m3": 1.0}),
    ("squat_safety_margin_m", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {**_DEEP_CHANNEL, "squat_safety_margin_m": 0.15}),
    # A Vessel Class (issue #94): the scheme places the vessel in a class, and the class fills the
    # draught and air-draft limits. The results of the same dimensions are those of typed ones.
    ("vessel_scheme", "bauxite", "box_hold", "self_propelled", 110.0, 11.4, 4.5,
     {"vessel_scheme": "CEMT-1992"}),
    ("vessel_class", "bauxite", "box_hold", "self_propelled", 100.0, 11.0, 4.5,
     {"vessel_class": "CEMT-Va", "length_overall_m": 108.0}),
    # The cost rates (issue #233), set to their defaults. They do not reach `simulate`: they feed the
    # capital budget, which is not part of the results hash.
    ("hull_steel_rate", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"hull_steel_rate": 2500.0}),
    ("tank_shell_rate", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"tank_shell_rate": 10000.0}),
    ("tank_supports_rate", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"tank_supports_rate": 2000.0}),
    ("insulation_rate", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"insulation_rate": 4000.0}),
    ("piping_rate", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"piping_rate": 4500.0}),
    ("outfitting_rate", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"outfitting_rate": 4000.0}),
    ("cargo_system_lump", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"cargo_system_lump": 500000.0}),
    ("paint_anodes_misc", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"paint_anodes_misc": 100000.0}),
    ("battery_rate_per_kwh", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"battery_rate_per_kwh": 250.0}),
    ("propulsion_rate_per_kw", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"propulsion_rate_per_kw": 1000.0}),
    ("wheelhouse_accom_lump", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"wheelhouse_accom_lump": 450000.0}),
    ("tug_outfitting_lump", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"tug_outfitting_lump": 200000.0}),
    ("yard_margin_fraction", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"yard_margin_fraction": 0.1}),
    ("delivery_lump", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, {"delivery_lump": 200000.0}),
    # The IS Code 2008 loading conditions (issue #426), opted in. A base from before them refuses the
    # inputs and runs the legacy conditions, so these cases show the change the opt-in makes.
    ("loading_conditions", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"loading_conditions": "is_code_2008"}),
    ("may_sail_without_ballast", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {"loading_conditions": "is_code_2008", "may_sail_without_ballast": False}),
    ("endurance_h", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"loading_conditions": "is_code_2008", "endurance_h": 120.0}),
    ("fixed_stores_t", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"loading_conditions": "is_code_2008", "fixed_stores_t": 30.0}),
    ("ballast_plan", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {"loading_conditions": "is_code_2008", "ballast_plan": "user_set",
      "ballast_departure_double_bottom_fill": 0.5, "ballast_departure_peak_fill": 1.0,
      "ballast_departure_wing_fill": 0.0, "ballast_arrival_double_bottom_fill": 1.0,
      "ballast_arrival_peak_fill": 0.0, "ballast_arrival_wing_fill": 0.98}),
    # The currency and the price base date (issue #429), set to their defaults: the same results as
    # leaving them out.
    ("currency", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"currency": "USD"}),
    ("price_base_date", "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0,
     {"price_base_date": None}),
    # The midship coefficient (issue #425), set to its default: the same results as leaving it out.
    ("midship_coefficient", "bauxite", "box_hold", "pushed_barge", 90.0, 14.0, 4.5,
     {"midship_coefficient": 1.0}),
    # The rules block (issue #274): the Service Area, and the Rule Sets stored with their content
    # hashes. Both are checked, not computed: the results do not move. A stale hash only warns. A base
    # from before them refuses the input and leaves it out.
    ("service_area", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"service_area": "coastal-and-river"}),
    ("rule_sets", "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5,
     {"rule_sets": [
         {"id": "is2008-a", "version": "1.0.0", "source": "default",
          "hash": "cfab2d1cda90d0cfda8edbf0955082345cd30ef2d102c9840e8d4feab5297e37"},
         {"id": "ovcd", "version": "2.1.0", "source": "default",
          "hash": "7a4b82e2b709ae694076787156efe9ad57b4916829085128330e4366e6e5295a"}]}),
    # The Hull Form parameters (issue #446), set to their defaults on a raked box. Inputs only until
    # the hydrostatics switch-over (#448): each shows no change. A base from before them refuses the
    # input and leaves it out.
    *((name, "bauxite", "sloped_hopper_hold", "self_propelled", 90.0, 14.0, 4.5, {name: value})
      for name, value in (("bow_rake_length_m", None), ("bow_rake_rise_m", None), ("bilge_radius_m", 0.0),
                          ("sheer_aft_m", 0.0), ("sheer_fwd_m", 0.0), ("camber_m", 0.0),
                          ("deadrise_deg", 5.0), ("keel_half_width_m", 0.5),
                          ("chine_half_breadth_m", None), ("chine_height_m", None),
                          ("shipd_parameters", None), ("hull_offsets", None))),
    # The liquid cargo tank inputs (issue #331): each moves a crude oil design.
    ("double_side_m", "crude_oil", "integral_tanks", "pushed_barge", 90.0, 14.0, 4.5, {"double_side_m": 1.2}),
    ("double_bottom_m", "crude_oil", "integral_tanks", "pushed_barge", 90.0, 14.0, 4.5, {"double_bottom_m": 0.9}),
    ("liquid_tanks_in_line", "crude_oil", "integral_tanks", "pushed_barge", 90.0, 14.0, 4.5,
     {"liquid_tanks_in_line": 6}),
    ("centreline_bulkhead", "crude_oil", "integral_tanks", "pushed_barge", 90.0, 14.0, 4.5,
     {"centreline_bulkhead": False}),
    ("cofferdam_m", "crude_oil", "integral_tanks", "pushed_barge", 90.0, 14.0, 4.5, {"cofferdam_m": 1.0}),
    ("pump_arrangement", "crude_oil", "integral_tanks", "pushed_barge", 90.0, 14.0, 4.5,
     {"pump_arrangement": "pump_room"}),
    ("pump_room_length_m", "crude_oil", "integral_tanks", "pushed_barge", 90.0, 14.0, 4.5,
     {"pump_arrangement": "pump_room", "pump_room_length_m": 5.0}),
    ("slop_share", "crude_oil", "integral_tanks", "pushed_barge", 90.0, 14.0, 4.5, {"slop_share": 0.03}),
    ("heating_kw_per_m3", "crude_oil", "integral_tanks", "pushed_barge", 90.0, 14.0, 4.5,
     {"heating_kw_per_m3": 0.3}),
    ("liquid_fill_ratio", "crude_oil", "integral_tanks", "pushed_barge", 90.0, 14.0, 4.5,
     {"liquid_fill_ratio": 0.9}),
)


def cases() -> dict[str, dict]:
    """Case name -> typed inputs of one Option. Everything else comes from the Defaults Profile,
    so a change to the profile moves these designs too.

    The set reaches every cargo, every optional input, dry bulk with a channel, gas designs that
    pass every check and gas designs that carry nothing. `tests/test_design_impact_cases.py` checks
    that it keeps doing so when an input or a cargo is added.
    """
    out: dict[str, dict] = {}
    cal = yaml.safe_load((ROOT / "calibration" / "cases.yaml").read_text(encoding="utf-8"))
    for c in cal["dry_barge"]:
        if c["hold_type"] != "box":
            continue                              # the Design format has no container hold yet
        extra = {"draught_limit_m": c["draught_m"]} if c.get("draught_m") else {}
        out[f"calibration {c['id']}"] = _inputs(
            "bauxite", "box_hold", "pushed_barge", c["loa_m"], c["beam_m"], c["depth_m"], **extra)
    motor = yaml.safe_load((ROOT / "calibration" / "cn_yangtze_gb38030_draft_2018.yaml").read_text(
        encoding="utf-8"))
    for cargo in ("bulk", "liquid"):
        point = motor["design_points"][cargo]
        for i, (loa, boa, draught, *_rest) in enumerate(point["rows"][:3]):
            knots = round(point["speed_kmh"] / 1.852, 1)
            depth = round(draught + CN_DEPTH_ABOVE_DRAUGHT_M, 2)
            if cargo == "bulk":
                inp = _inputs("bauxite", "sloped_hopper_hold", "self_propelled", loa, boa,
                              depth, draught_limit_m=draught, max_speed_kn=knots)
            else:
                inp = _inputs("LPG_propane", "type_c_tanks", "self_propelled", loa, boa,
                              depth, draught_limit_m=draught, max_speed_kn=knots)
            out[f"sweep CN-derived {cargo} {i + 1} ({loa:g}x{boa:g})"] = inp
    # A sweep over vessel kinds, sizes, formations and a channel.
    for loa, beam in ((60.0, 10.0), (90.0, 14.0)):
        tag = f"{loa:g}x{beam:g}"
        out[f"sweep box {tag}"] = _inputs("bauxite", "box_hold", "pushed_barge", loa, beam, 4.5)
        out[f"sweep box self-propelled {tag}"] = _inputs(
            "bauxite", "box_hold", "self_propelled", loa, beam, 4.5)
        out[f"sweep hopper {tag}"] = _inputs(
            "bauxite", "sloped_hopper_hold", "self_propelled", loa, beam, 4.5)
        out[f"sweep hopper pushed {tag}"] = _inputs(
            "bauxite", "sloped_hopper_hold", "pushed_barge", loa, beam, 4.5)
        out[f"sweep box channel {tag}"] = _inputs(
            "bauxite", "box_hold", "pushed_barge", loa, beam, 4.5, **_CHANNEL)
        out[f"sweep hopper channel {tag}"] = _inputs(
            "bauxite", "sloped_hopper_hold", "self_propelled", loa, beam, 4.5, max_speed_kn=9.0,
            channel_current_kn=1.5, **_CHANNEL)
        # `IGC_2G_EXACT` is not offered since issue #237: a file that names it is read as `IGC_2G`
        # (with a warning). The cases keep that name so that a base from before #237 and this tree
        # run the same designs and the table shows what moved (the stepped IGC_2G cases moved most).
        for ruleset in ("IGC_2G", "IGC_2G_EXACT"):
            out[f"sweep gas {ruleset} {tag}"] = _inputs(
                "LPG_propane", "type_c_tanks", "self_propelled", loa, beam, 5.5, ruleset=ruleset)
        out[f"sweep gas channel {tag}"] = _inputs(
            "LNG", "type_c_tanks", "pushed_barge", loa, beam, 6.0, draught_limit_m=3.5,
            channel_depth_m=5.0, channel_bottom_width_m=60.0, optimise_depth=True)
    # Every gas cargo (the sweep above uses only propane and LNG).
    for cargo in GAS_CARGOES:
        out[f"sweep gas cargo {cargo}"] = _inputs(
            cargo, "type_c_tanks", "self_propelled", 90.0, 14.0, 5.5, max_speed_kn=9.0)
        out[f"sweep gas cargo {cargo} pushed"] = _inputs(
            cargo, "type_c_tanks", "pushed_barge", 90.0, 14.0, 6.0)
    # Each LPG Cargo in each carriage condition (issue #292): a different row of the tank default tables.
    for cargo in ("LPG_propane", "LPG_butane", "LPG_mix"):
        for carriage in ("fully_pressurised", "semi_pressurised", "fully_refrigerated"):
            out[f"sweep gas carriage {cargo} {carriage}"] = _inputs(
                cargo, "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, max_speed_kn=9.0,
                lpg_carriage_condition=carriage)
    # Gas designs that pass every check, so a change to a check's wording or limit shows.
    out["sweep gas passes all checks IGC_2G 110x11.4"] = _inputs(
        "LPG_propane", "type_c_tanks", "self_propelled", 110.0, 11.4, 6.0, ruleset="IGC_2G")
    out["sweep gas passes all checks IGC_2G_EXACT 80x14"] = _inputs(
        "LPG_propane", "type_c_tanks", "self_propelled", 80.0, 14.0, 4.5, ruleset="IGC_2G_EXACT")
    # Wide hulls, where the tank outer diameter cap sets the tank (12 m until issue #175, 16 m now).
    for ruleset in ("IGC_2G", "IGC_2G_EXACT"):
        out[f"sweep gas wide hull {ruleset} 110x24"] = _inputs(
            "LNG", "type_c_tanks", "self_propelled", 110.0, 24.0, 6.5, ruleset=ruleset,
            draught_limit_m=4.5)
    # A long hull: two tanks along it are longer than 7 diameters, which is refused. The ruleset is
    # IGC_2G so that no other error check fails and the length check is the one that refuses it.
    out["sweep gas tank longer than 7 diameters IGC_2G 200x12"] = _inputs(
        "LNG", "type_c_tanks", "self_propelled", 200.0, 12.0, 5.5, ruleset="IGC_2G")
    # Designs that carry nothing: the messages are results too.
    out["sweep gas carries nothing (hull too short)"] = _inputs(
        "LPG_propane", "type_c_tanks", "self_propelled", 12.0, 4.0, 2.0)
    out["sweep gas carries nothing (draught limit)"] = _inputs(
        "LPG_propane", "type_c_tanks", "self_propelled", 90.0, 14.0, 5.5, draught_limit_m=0.5)
    out["sweep box lightship over the draught limit"] = _inputs(
        "bauxite", "box_hold", "pushed_barge", 60.0, 10.0, 4.5, draught_limit_m=0.3)
    # A left-out moulded depth is the Cargo's ratio times the beam (issue #120): one case for each Cargo
    # group the format has. A base from before the rule has no such default, so these show as new.
    for cargo, containment, group in (("bauxite", "box_hold", "dry bulk"), ("LNG", "type_c_tanks", "gas"),
                                      ("crude_oil", "integral_tanks", "liquid")):
        out[f"sweep default depth {group} 90x14"] = _inputs(
            cargo, containment, "self_propelled", 90.0, 14.0, None)
    # Crude oil in integral tanks (spec 0006, issue #331): a pushed and a self-propelled hull at two
    # sizes, one with a channel, one with a tank barge on coastal service (slop tanks), and one that
    # carries nothing.
    for loa, beam in ((60.0, 10.0), (90.0, 14.0)):
        tag = f"{loa:g}x{beam:g}"
        out[f"sweep crude pushed {tag}"] = _inputs(
            "crude_oil", "integral_tanks", "pushed_barge", loa, beam, 4.5)
        out[f"sweep crude self-propelled {tag}"] = _inputs(
            "crude_oil", "integral_tanks", "self_propelled", loa, beam, 4.5)
    out["sweep crude channel 90x14"] = _inputs(
        "crude_oil", "integral_tanks", "pushed_barge", 90.0, 14.0, 4.5, **_CHANNEL)
    out["sweep crude coastal service 90x14"] = _inputs(
        "crude_oil", "integral_tanks", "self_propelled", 90.0, 14.0, 4.5, service_area="coastal-and-river")
    out["sweep crude carries nothing (draught limit)"] = _inputs(
        "crude_oil", "integral_tanks", "pushed_barge", 90.0, 14.0, 4.5, draught_limit_m=0.3)
    # One case for each optional input.
    for name, cargo, containment, formation, loa, beam, depth, extra in OPTIONAL_INPUT_CASES:
        out[f"sweep optional {name}"] = _inputs(cargo, containment, formation, loa, beam, depth, **extra)
    return out


# --- running ovcd -------------------------------------------------------------------------------

def run(cmd, **kw) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def fill_file(binary: Path, raw: Path, filled: Path) -> subprocess.CompletedProcess:
    """`ovcd fill` of one case, as a new Design: an input added after the first files takes the
    Defaults Profile value (`--as-new`, issue #292). Without it `fill` would give such an input the
    fixed value an old file gets, and a change to the profile could not show. A base from before the
    flag does not know it and fills as it always did."""
    fill = run([str(binary), "fill", "--as-new", str(raw), str(filled)])
    if fill.returncode != 0 and "unknown option" in fill.stderr:
        fill = run([str(binary), "fill", str(raw), str(filled)])
    return fill


def run_case(binary: Path, name: str, typed: dict, tmp: Path) -> dict:
    """One case through one binary: {"results": {...}} or {"error": "..."}."""
    raw, filled = tmp / "in.vdesign", tmp / "filled.vdesign"
    filled.unlink(missing_ok=True)
    # The newest format the binary reads: version 2 (issue #175), or version 1 for a base that
    # predates it. A version 1 file is changed to the current inputs as it is read, so the
    # head must be given version 2: version 1 would give a gas design the 12 m tank diameter cap
    # that version 1 files had, not the Defaults Profile value.
    typed = dict(typed)
    for version in (2, 1):
        while True:
            filled.unlink(missing_ok=True)
            raw.write_text(json.dumps({"format": "vdesign", "format_version": version,
                                       "engine_version": "0.0.0",
                                       "options": [{"name": name, "inputs": typed}]}), encoding="utf-8")
            fill = fill_file(binary, raw, filled)
            # A binary from before an input existed refuses it by name. Leave it out: that binary
            # then uses the fixed value it always used, which the case sets the input to.
            unknown = re.search(r"unknown input `([a-z0-9_]+)`", fill.stderr)
            if fill.returncode != 0 and unknown and unknown.group(1) in typed:
                del typed[unknown.group(1)]
                continue
            break
        if fill.returncode == 0 or "format_version" not in fill.stderr:
            break
    if fill.returncode != 0:
        return {"error": "fill: " + " | ".join(fill.stderr.strip().splitlines()[-3:])}
    res = run([str(binary), "run", "--json", str(filled)])
    try:
        option = json.loads(res.stdout)["options"][0]
    except (ValueError, KeyError, IndexError):
        return {"error": "run: " + " | ".join((res.stderr or res.stdout).strip().splitlines()[-3:])}
    if option.get("results") is None:
        return {"error": option.get("message", "no results")}
    return {"results": option["results"]}


def run_all(binary: Path, case_set: dict[str, dict]) -> dict[str, dict]:
    with tempfile.TemporaryDirectory() as t:
        return {name: run_case(binary, name, typed, Path(t)) for name, typed in case_set.items()}


# --- comparing ----------------------------------------------------------------------------------

def leaves(value, path="") -> dict[str, object]:
    """Every scalar of a JSON value, by path."""
    if isinstance(value, dict):
        out: dict[str, object] = {}
        for k, v in value.items():
            out.update(leaves(v, f"{path}.{k}" if path else k))
        return out
    if isinstance(value, list):
        out = {}
        for i, v in enumerate(value):
            out.update(leaves(v, f"{path}[{i}]"))
        return out
    return {path: value}


def show(v) -> str:
    if v is None:
        return "none"
    if isinstance(v, float):
        return f"{v:.12g}"          # more than the 1e-9 tolerance needs, so a 1e-6 move shows
    return str(v)


def change(a, b) -> str:
    if is_number(a) and is_number(b) and isinstance(a, float) != isinstance(b, float) and a == b:
        return f"{type(a).__name__} to {type(b).__name__}"
    if isinstance(a, (int, float)) and isinstance(b, (int, float)) \
            and not isinstance(a, bool) and not isinstance(b, bool):
        if a == 0:
            return f"{b - a:+.6g}"
        return f"{(b - a) / abs(a) * 100:+.3g} %"
    return "changed"


def is_number(v) -> bool:
    return isinstance(v, (int, float)) and not isinstance(v, bool)


def same(a, b) -> bool:
    """Equal, with numbers compared at the project tolerance (relative and absolute 1e-9).
    A change of type, int to float or back, is a change: it changes the results hash."""
    if isinstance(a, bool) or isinstance(b, bool):
        return a == b and type(a) is type(b)
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        if isinstance(a, float) != isinstance(b, float):
            return False
        return math.isclose(a, b, rel_tol=1e-9, abs_tol=1e-9)
    return a == b


def size(row: dict) -> float:
    """How far a row moved, for ordering: relative for numbers, infinite for anything else."""
    a, b = row["before"], row["after"]
    if isinstance(a, (int, float)) and isinstance(b, (int, float)) \
            and not isinstance(a, bool) and not isinstance(b, bool):
        return abs(b - a) / abs(a) if a else abs(b - a)
    return math.inf


def compare(before: dict[str, dict], after: dict[str, dict]) -> list[dict]:
    """Rows of what moved, biggest first: case, quantity, before, after, change. A number that
    moved by no more than the tolerance (1e-9) has not moved."""
    rows = []
    for name in before:
        a, b = before[name], after.get(name, {"error": "case missing"})
        if "error" in a or "error" in b:
            if a != b:
                rows.append({"case": name, "quantity": "result",
                             "before": a.get("error", "computed"), "after": b.get("error", "computed"),
                             "change": "changed"})
            continue
        # The UNHASHED_KEYS are not results and are not in the results hash (docs/adr/0007): a new
        # key there is not a design change.
        la = leaves({k: v for k, v in a["results"].items() if k not in UNHASHED_KEYS})
        lb = leaves({k: v for k, v in b["results"].items() if k not in UNHASHED_KEYS})
        for path in sorted(set(la) | set(lb)):
            va, vb = la.get(path, "absent"), lb.get(path, "absent")
            if not same(va, vb):
                rows.append({"case": name, "quantity": path, "before": va, "after": vb,
                             "change": change(va, vb)})
    return sorted(rows, key=lambda r: (-size(r), r["case"], r["quantity"]))


def table(rows: list[dict]) -> str:
    shown = rows[:MAX_ROWS]
    lines = ["| Case | Quantity | Before | After | Change |", "|---|---|---|---|---|"]
    for r in shown:
        cell = lambda s: str(s).replace("|", "\\|").replace("\n", " ")
        lines.append(f"| {cell(r['case'])} | `{cell(r['quantity'])}` | {cell(show(r['before']))} "
                     f"| {cell(show(r['after']))} | {cell(r['change'])} |")
    if len(rows) > len(shown):
        lines.append(f"| ... | {len(rows) - len(shown)} more changed quantities | | | |")
    return "\n".join(lines)


def report_text(total: int, rows: list[dict], base: str, head: str) -> tuple[str, str]:
    """(the report, the bare table or "")."""
    moved = sorted({r["case"] for r in rows})
    if not rows:
        return (f"## Design impact\n\nno design change: {total} case designs give identical results "
                f"on `{base}` and on `{head}`.\n\n{DEPTH_NOTE}\n"), ""
    t = table(rows)
    return (f"## Design impact\n\n**design change**: {len(moved)} of {total} case designs moved "
            f"({len(rows)} quantities) between `{base}` and `{head}`. Labelled `{LABEL}`; no "
            f"sign-off is needed.\n\n{t}\n\n{DEPTH_NOTE}\n"), t


# --- finding and building the two binaries ------------------------------------------------------

def git(*args, check=True) -> str:
    r = run(["git", *args], cwd=ROOT)
    if check and r.returncode != 0:
        raise SystemExit(f"design_impact: git {' '.join(args)} failed: {r.stderr.strip()}")
    return r.stdout.strip()


def base_commit() -> tuple[str, bool]:
    """(the commit to compare with, whether this tree is ahead of main). Fails if there is none."""
    head = git("rev-parse", "HEAD")
    for ref in ("origin/main", "main"):
        if run(["git", "rev-parse", "--verify", "-q", ref], cwd=ROOT).returncode == 0:
            mb = git("merge-base", "HEAD", ref)
            if mb != head:
                return mb, True
            break
    parent = run(["git", "rev-parse", "--verify", "-q", "HEAD~1"], cwd=ROOT)
    if parent.returncode != 0:
        raise SystemExit("design_impact: no base commit to compare with (shallow clone? "
                         "CI must fetch the full history)")
    return parent.stdout.strip(), False


def build(tree: Path, target: Path, fresh: bool = False) -> Path:
    """Build `ovcd` in `tree` and return the binary that cargo says it built.

    The path comes from cargo's own JSON messages, never from a guess about the target directory.
    `fresh` starts from an empty target directory, and then the binary must be newer than the start
    of the build. The check fails when cargo reports no `ovcd` binary, or the file is missing or
    is not under `target`: a leftover binary from another tree must never be measured.
    """
    if fresh:
        shutil.rmtree(target, ignore_errors=True)
    started = time.time()
    env = {**os.environ, "CARGO_TARGET_DIR": str(target)}
    r = subprocess.run(["cargo", "build", "-q", "-p", "ovcd-cli", "--message-format=json-render-diagnostics"],
                       cwd=tree, env=env, stdout=subprocess.PIPE, text=True)
    if r.returncode != 0:
        raise SystemExit(f"design_impact: cargo build failed in {tree}")
    exe = None
    for line in r.stdout.splitlines():
        try:
            msg = json.loads(line)
        except ValueError:
            continue
        if (isinstance(msg, dict) and msg.get("reason") == "compiler-artifact" and msg.get("executable")
                and msg.get("target", {}).get("name") == "ovcd" and "bin" in msg["target"].get("kind", [])):
            exe = Path(msg["executable"])
    if exe is None:
        raise SystemExit(f"design_impact: cargo built in {tree} but reported no `ovcd` binary")
    if not exe.is_file():
        raise SystemExit(f"design_impact: cargo reported {exe} but the file is missing")
    if not exe.resolve().is_relative_to(target.resolve()):
        raise SystemExit(f"design_impact: the `ovcd` binary {exe} is not under {target}")
    if fresh and exe.stat().st_mtime < started - 2:
        raise SystemExit(f"design_impact: {exe} is older than this build (stale binary)")
    return exe


def case_count_at(tree: Path) -> int | None:
    """How many cases the check at `tree` (the base commit) runs: its own `cases()` is loaded and
    counted. None when that commit has no check yet. A base that cannot be counted fails the gate."""
    path = tree / "tools" / "design_impact.py"
    if not path.is_file():
        return None
    try:
        spec = importlib.util.spec_from_file_location("design_impact_base", path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return len(module.cases())
    except Exception as e:                                       # noqa: BLE001 - any failure is a failure
        raise SystemExit(f"design_impact: cannot count the cases of the base commit: {e!r}") from e


def measure() -> tuple[str, str, int, list[dict], bool, int | None]:
    """Run both binaries. (base label, head label, case count, moved rows, ahead of main, the
    base commit's case count)."""
    base_sha, ahead = base_commit()
    case_set = cases()
    work = Path(tempfile.mkdtemp(prefix="design-impact-"))
    try:
        base_tree = work / "base"
        git("worktree", "add", "-q", "--detach", str(base_tree), base_sha)
        try:
            base_total = case_count_at(base_tree)
            base_bin = build(base_tree, ROOT / "target" / "design-impact-base", fresh=True)
            head_bin = build(ROOT, ROOT / "target")
            before, after = run_all(base_bin, case_set), run_all(head_bin, case_set)
            # Fail closed: a check that computed nothing on either side proves nothing.
            for label, side in (("base", before), ("this tree", after)):
                if not any("results" in v for v in side.values()):
                    raise SystemExit(f"design_impact: no case designs computed on the {label} binary")
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", str(base_tree)], cwd=ROOT,
                           capture_output=True)
    finally:
        shutil.rmtree(work, ignore_errors=True)
    base_label = ("main " if ahead else "the parent commit ") + base_sha[:7]
    head_label = "this branch " + git("rev-parse", "--short", "HEAD")
    return base_label, head_label, len(case_set), compare(before, after), ahead, base_total


# --- outputs ------------------------------------------------------------------------------------

def write_outputs(text: str, rows: list[dict]) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "report.md").write_text(text, encoding="utf-8")
    (OUT / "summary.json").write_text(json.dumps(
        {"moved": bool(rows), "quantities": len(rows), "cases": sorted({r["case"] for r in rows})},
        indent=1), encoding="utf-8")
    print(text)
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as f:
            f.write(text + "\n")


def publish(text: str, moved: bool, pr: int) -> None:
    body = json.loads(subprocess.run(["gh", "pr", "view", str(pr), "--json", "body"], cwd=ROOT,
                                     capture_output=True, text=True, check=True).stdout)["body"] or ""
    block = f"{START}\n{text.rstrip()}\n{END}"
    pattern = re.compile(re.escape(START) + r".*?" + re.escape(END), re.S)
    new = pattern.sub(lambda _m: block, body) if pattern.search(body) else (body.rstrip() + "\n\n" + block + "\n")
    OUT.mkdir(parents=True, exist_ok=True)
    path = OUT / "body.md"
    path.write_text(new, encoding="utf-8")
    subprocess.run(["gh", "pr", "edit", str(pr), "--body-file", str(path)], cwd=ROOT, check=True)
    if moved:
        subprocess.run(["gh", "label", "create", LABEL, "--color", "fbca04", "--description",
                        "Moves design results; tracked, no sign-off"], cwd=ROOT, capture_output=True)
        subprocess.run(["gh", "pr", "edit", str(pr), "--add-label", LABEL], cwd=ROOT, check=True)
    else:
        subprocess.run(["gh", "pr", "edit", str(pr), "--remove-label", LABEL], cwd=ROOT,
                       capture_output=True)


# --- publishing a report built by an untrusted run (fork pull requests) --------------------------
# The `Design impact` workflow runs the pull request's own code with a read-only token and keeps
# report.md, summary.json and pr.txt as an artifact. A second workflow (`workflow_run`, with a write
# token, running THIS file from the default branch) calls `publish-artifact`. The artifact is data
# from an untrusted run: it is size-limited, checked for shape and made harmless before use, and it
# must belong to the pull request that the run was started for.

MAX_REPORT_CHARS = 60_000                    # a pull request body is limited to 65,536
ARTIFACT_FILES = ("report.md", "summary.json", "pr.txt")


def load_artifact(directory: Path) -> tuple[str, bool, int]:
    """(report text, whether designs moved, pull request number) from an untrusted artifact.
    Raises SystemExit on anything that is not exactly the expected shape."""
    def bad(why: str):
        raise SystemExit(f"design_impact: artifact rejected: {why}")

    for name in ARTIFACT_FILES:
        f = directory / name
        if f.is_symlink() or not f.is_file():
            bad(f"{name} is missing or is not a plain file")
        if f.stat().st_size > 4 * MAX_REPORT_CHARS:
            bad(f"{name} is too large")
    raw_pr = (directory / "pr.txt").read_text(encoding="utf-8").strip()
    if not re.fullmatch(r"[1-9][0-9]{0,8}", raw_pr):
        bad("pr.txt is not a pull request number")
    try:
        summary = json.loads((directory / "summary.json").read_text(encoding="utf-8"))
    except ValueError:
        bad("summary.json is not JSON")
    if not isinstance(summary, dict) or not isinstance(summary.get("moved"), bool):
        bad("summary.json has no boolean `moved`")
    text = (directory / "report.md").read_text(encoding="utf-8")
    if len(text) > MAX_REPORT_CHARS:
        bad("report.md is too long")
    if not text.startswith("## Design impact\n"):
        bad("report.md does not start with the Design impact heading")
    return harmless(text), summary["moved"], int(raw_pr)


def harmless(text: str) -> str:
    """The report is written by the pull request's code, so it may not carry markers that would
    break out of its block, control characters, or an @mention that notifies someone."""
    text = text.replace(START, "").replace(END, "")
    text = "".join(c for c in text if c in "\n\t" or ord(c) >= 32)
    return re.sub(r"@(?=[A-Za-z0-9_])", "@\u200b", text)


def publish_artifact(directory: Path, head_sha: str, head_repo: str) -> None:
    """Publish the report in `directory` to the pull request it names, if that pull request's head is
    the commit (and repository) that the workflow run was made for. Needs GH_TOKEN."""
    text, moved, pr = load_artifact(directory)
    repo = os.environ.get("GITHUB_REPOSITORY") or ""
    if not re.fullmatch(r"[\w.-]+/[\w.-]+", repo):
        raise SystemExit("design_impact: GITHUB_REPOSITORY is not set")
    r = subprocess.run(["gh", "api", f"repos/{repo}/pulls/{pr}", "--jq",
                        '[.state, .head.sha, .head.repo.full_name // ""] | @tsv'],
                       cwd=ROOT, capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"design_impact: cannot read pull request {pr}: {r.stderr.strip()}")
    state, sha, repo_of_head = (r.stdout.strip().split("\t") + ["", "", ""])[:3]
    if state != "open":
        print(f"design_impact: pull request {pr} is {state}; nothing to publish")
        return
    if sha != head_sha or repo_of_head != head_repo:
        print(f"design_impact: pull request {pr} has moved on (head {sha[:7]}, run was for {head_sha[:7]}); "
              "a newer run will publish")
        return
    publish(text, moved, pr)


# --- the changelog ------------------------------------------------------------------------------

def changelog_has(tbl: str) -> bool:
    return CHANGELOG.exists() and tbl in CHANGELOG.read_text(encoding="utf-8")


def write_changelog(tbl: str, key: str, base: str, head: str) -> None:
    """Add (or replace, by `key`) one entry under "Design changes", newest first.

    Entries go after the `## Design changes` heading, whatever `### ` headings come before it.
    Anything after the section (the next `## ` heading) is kept as it was.
    """
    text = CHANGELOG.read_text(encoding="utf-8") if CHANGELOG.exists() else CHANGELOG_HEAD
    heading = re.search(rf"^{re.escape(SECTION)}[ \t]*$", text, re.M)
    if heading is None:                                   # no section yet: start it at the end
        intro_text = CHANGELOG_HEAD.split(SECTION, 1)[1]
        text = text.rstrip("\n") + f"\n\n{SECTION}{intro_text}"
        heading = re.search(rf"^{re.escape(SECTION)}[ \t]*$", text, re.M)
    before, section = text[:heading.end()], text[heading.end():]
    nxt = re.search(r"\n## ", section)
    tail = section[nxt.start():] if nxt else ""
    section = section[:nxt.start()] if nxt else section
    intro, _, rest = section.partition(ENTRIES_AT)
    entries = [e for e in (ENTRIES_AT + rest).split(ENTRIES_AT) if e.strip()] if rest else []
    entries = [e for e in entries if not e.startswith(f"{key} ")]
    today = datetime.date.today().isoformat()
    entry = f"{key} ({today})\n\nBetween `{base}` and `{head}`.\n\n{tbl}\n"
    body = intro.rstrip("\n") + "\n" + "".join(ENTRIES_AT + e.rstrip("\n") + "\n" for e in [entry] + entries)
    CHANGELOG.write_text(before + body + ("\n" + tail.lstrip("\n") if tail else ""), encoding="utf-8")


def branch_key() -> str:
    name = git("rev-parse", "--abbrev-ref", "HEAD", check=False)
    return name if name and name != "HEAD" else "branch " + git("rev-parse", "--short", "HEAD")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("mode", choices=["report", "gate", "changelog", "publish-artifact"])
    ap.add_argument("artifact_dir", nargs="?", type=Path, help="publish-artifact: the downloaded artifact")
    ap.add_argument("--head-sha", help="publish-artifact: the commit the workflow run was made for")
    ap.add_argument("--head-repo", help="publish-artifact: the repository that commit is in")
    ap.add_argument("--publish", action="store_true", help="report: put the report in the PR body and set the label")
    ap.add_argument("--pr", type=int, help="the pull request number for --publish")
    ap.add_argument("--key", help="changelog entry name (default: the branch)")
    args = ap.parse_args()
    if args.mode == "publish-artifact":
        if not (args.artifact_dir and args.head_sha and args.head_repo):
            raise SystemExit("design_impact: publish-artifact needs DIR, --head-sha and --head-repo")
        publish_artifact(args.artifact_dir, args.head_sha, args.head_repo)
        return 0
    base, head, total, rows, ahead, base_total = measure()
    text, tbl = report_text(total, rows, base, head)
    write_outputs(text, rows)
    if args.publish:
        if not args.pr:
            raise SystemExit("design_impact: --publish needs --pr")
        publish(text, bool(rows), args.pr)
    if args.mode == "changelog":
        if not rows:
            print("design_impact: no design change; nothing to write to CHANGELOG.md")
            return 0
        write_changelog(tbl, args.key or branch_key(), base, head)
        print(f"design_impact: wrote CHANGELOG.md ({args.key or branch_key()})")
        return 0
    if args.mode == "gate" and base_total is not None and total < base_total:
        print(f"design_impact: this branch runs {total} cases but the base commit runs {base_total}. "
              "Removing cases weakens the check; keep every case, or ask a maintainer.", file=sys.stderr)
        return 1
    if args.mode == "gate" and rows and ahead and not changelog_has(tbl):
        print("design_impact: this branch moves designs but CHANGELOG.md does not hold the table above.\n"
              "  Run: python3 tools/design_impact.py changelog   and commit CHANGELOG.md", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
