<!-- unreleased -->
- Calibration (issue #359, part 2): the pushed Formation allowance is fitted. `calibration/formation_cases.yaml` holds two pushers with a published installed power and a published convoy: the SDP 4600 (16 barges of 61 m x 15 m, 3,000 t each; in-sample) and the CDS 4115 (12 barges; held out). `python3 tools/formation_calibration.py <ovcd> --write` runs each as a pushed convoy, fits `formation_resistance_allowance` and writes `calibration/formation_results.md` and `calibration/formation_factors.yaml`. The fitted value is 1.6 (an Estimate): the modelled pusher power is 5,372 kW for the published 5,369 kW (in-sample), and 3,497 kW for the published 3,990 kW of the held-out case (-12.4 %, inside its 30 % band). The sources print no convoy speed for most pushers, so both cases run at 7 knots (the CDS 4115 design speed, published; assumed for the SDP 4600), and the barge sizes of the CDS 4115 are derived from its convoy size. Three more records (RApide 4600-Z3 and two others) cannot run yet and are listed with the reason. The Defaults Profile `formation_resistance_allowance` is 1.6 for a pushed barge and a pushed convoy (was 1.00) with the soft limit raised from 1.5 to 2.0; an ATB stays 1.05 and a towed barge 1.00. A new pushed Design that includes its pusher gets a larger pusher; a file that stores the input keeps its value.

<!-- design-impact -->
### ticket/359-formation-calibration (2026-10-22)

Between `main f9f99bb` and `this branch 80ac506`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `formation.coupling_force_kn` | absent | 208.008076553 | changed |
| calibration amaggi-box-2000 | `formation.design_speed_kn` | absent | 9.7 | changed |
| calibration amaggi-box-2000 | `formation.environments[0].name` | absent | Deep water | changed |
| calibration amaggi-box-2000 | `formation.environments[0].power_at_service_kw` | absent | 396.508026295 | changed |
| calibration amaggi-box-2000 | `formation.environments[0].service_speed_kn` | absent | 9.7 | changed |
| calibration amaggi-box-2000 | `formation.equivalent_hull.beam_m` | absent | 10.67 | changed |
| calibration amaggi-box-2000 | `formation.equivalent_hull.block_coefficient` | absent | 0.824081626236 | changed |
| calibration amaggi-box-2000 | `formation.equivalent_hull.draught_m` | absent | 3.56 | changed |
| calibration amaggi-box-2000 | `formation.equivalent_hull.length_m` | absent | 70.4282153284 | changed |
| calibration amaggi-box-2000 | `formation.method_status` | absent | unverified | changed |
| calibration amaggi-box-2000 | `formation.notes[0]` | absent | The Formation's resistance is that of its Equivalent Hull (the barges and the pusher as one hull) by the single-hull method, times the Formation resistance allowance. The method is unverified. | changed |
| calibration amaggi-box-2000 | `formation.notes[1]` | absent | Coupling force between the pusher and the pushed craft: 208 kN (ES-TRIN 2025/1, ESI-III-6 2.1: 270 x P_B x L_S / B_S, with P_B 476 kW, L_S 9.5 m taken as the pusher length, B_S 5.8 m). | changed |
| calibration amaggi-box-2000 | `formation.resistance_allowance` | absent | 1.6 | changed |
| calibration amaggi-box-2000 | `formation.sizing.converged` | absent | True | changed |
| calibration amaggi-box-2000 | `formation.sizing.max_passes` | absent | 5 | changed |
| calibration amaggi-box-2000 | `formation.sizing.passes` | absent | 4 | changed |
| calibration amaggi-box-2000 | `formation.sizing.tolerance` | absent | 0.01 | changed |
| calibration amaggi-box-2000 | `formation.tug.beam_m` | absent | 5.84769780488 | changed |
| calibration amaggi-box-2000 | `formation.tug.bollard_pull_t` | absent | 12.1158636493 | changed |
| calibration amaggi-box-2000 | `formation.tug.depth_m` | absent | 3 | changed |
| calibration amaggi-box-2000 | `formation.tug.draught_m` | absent | 2 | changed |
| calibration amaggi-box-2000 | `formation.tug.hull_steel_t` | absent | 16.6101785976 | changed |
| calibration amaggi-box-2000 | `formation.tug.installed_power_kw` | absent | 475.809631554 | changed |
| calibration amaggi-box-2000 | `formation.tug.installed_power_sized` | absent | True | changed |
| calibration amaggi-box-2000 | `formation.tug.length_m` | absent | 9.46821532841 | changed |
| calibration amaggi-box-2000 | `formation.tug.lightship_t` | absent | 40.3630059355 | changed |
| calibration amaggi-box-2000 | `formation.tug.machinery_t` | absent | 23.7528273379 | changed |
| calibration amaggi-box-2000 | `formation.tug.propeller_diameter_m` | absent | 1.6 | changed |
| calibration amaggi-box-2000 | `formation.tug.propulsor_count` | absent | 2 | changed |
| calibration amaggi-box-2000 | `formation.tug.propulsor_type` | absent | shaft_nozzle | changed |
| calibration amaggi-box-2000 | `formation.tug_included` | absent | True | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.coupling_force_kn` | absent | 128.118211611 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.design_speed_kn` | absent | 9.7 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.environments[0].name` | absent | Deep water | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.environments[0].power_at_service_kw` | absent | 266.090577557 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.environments[0].service_speed_kn` | absent | 9.7 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.equivalent_hull.beam_m` | absent | 7.62 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.equivalent_hull.block_coefficient` | absent | 0.862625418891 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.equivalent_hull.draught_m` | absent | 2.7432 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.equivalent_hull.length_m` | absent | 68.2517967898 | changed |
| ... | 3097 more changed quantities | | | |
