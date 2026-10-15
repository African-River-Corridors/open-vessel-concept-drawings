# This repo is the single master of the vessel code

The vessel core was ported from a prior engine (ADR 0002). Both copies then lived on for a while.
This repo is now the **single master**: the other copy will be retired, and code that needs vessel
calculations imports this package.

The two copies were reconciled after the port. Nothing changed in the other copy after the port, so
the only work was to bring over the vessel modules that had been left behind.

## Drifted modules

| Module | Difference | Decision |
|---|---|---|
| `hull` | none | keep |
| `hydrostatics` | none | keep |
| `journey_derating` | none | keep |
| `models`, `checks`, `tanks`, `weights` | none | keep |
| `barge_capacity` | here only: `hold_type` + `lightship_k_t_per_m3` (calibration pass 1, box/container holds); `IGC_2G_EXACT` clearance loop (`d_vc_exact`); `gas_basis` on the result (feeds `gas_stability`) | keep — intentional improvements. All new fields default to the old behaviour, so existing callers get the same numbers. |
| `protective_location` | here only: the `IGC_2G_EXACT` ruleset, `d_vc_exact` (the `ADN_G` ruleset it also had was removed in issue #175: the engine applies IGC rules only to gas tanks); `required_bottom_protection` folded into the clearance functions | keep — the "IGC_2G" reading is unchanged. No caller outside the module used `required_bottom_protection`. |

## Modules brought over in the reconciliation

| Module | Decision |
|---|---|
| `vessel_standards` | **ported** with its tests. Same names (`VESSEL_STANDARDS`, `VesselStandard`, `get_standard`, `standards_for`, `validate_catalogue`). Since issue #94 the data is every class of `catalogue/vessel_classes.yaml`, generated into the core by `tools/gen_vessel_classes.py`. |
| `ga_adapter` | **ported** (`to_ga`, `is_gas`), with new tests. Depends only on `models` and `barge_capacity`. |

## Consequences

- `vessel_standards` overlapped `catalogue/`. Since issue #94 it does not: `catalogue/vessel_classes.yaml`
  is the one source, the core's list is generated from it, and a test fails on any drift. The ids of the
  12 classes held by hand before still work as aliases.
- `CN_II_motor.moulded_depth_m = 6.5` was an example design value, not a published class value. Issue
  #94 dropped it: no class sets a moulded depth, and depth stays typed.
- `ga_adapter` builds the structures a GA drawer reads. This repo has no SVG GA drawer (ADR 0003); the
  adapter is here so callers keep one import path.
