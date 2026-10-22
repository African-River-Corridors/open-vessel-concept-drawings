<!-- unreleased -->
- Feature (issue #359, part 1): a pushed Design that includes its pusher (`tug_included`, true for a new Design) gains a `formation` block in its results. The pusher is sized for the Formation in a loop of at most 5 passes (tolerance 1 %): its dimensions, machinery and steel come from the power it needs, and the power comes from the Equivalent Hull, which is the barges and the pusher taken as one hull. The block gives the power and the service speed in each environment, the bollard pull and the coupling force (ES-TRIN 2025/1 ESI-III-6 2.1, with the pusher length as L_S and the pusher beam as B_S). All of it is uncalibrated: the label is in the unhashed key `calibration.formation`, and the pusher depth (draught + 1.0 m) and the 0.5 m double bottom of its machinery items are Estimates. The fixture hashes of the pushed Designs moved because they gain the block. The calibration against built pushers is part 2.

<!-- design-impact -->
### ticket/359-formation-powering-1 (2026-10-22)

Between `main f9f99bb` and `this branch 74b6e30`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `formation.coupling_force_kn` | absent | 99.4132392689 | changed |
| calibration amaggi-box-2000 | `formation.design_speed_kn` | absent | 9.7 | changed |
| calibration amaggi-box-2000 | `formation.environments[0].name` | absent | Deep water | changed |
| calibration amaggi-box-2000 | `formation.environments[0].power_at_service_kw` | absent | 216.156116384 | changed |
| calibration amaggi-box-2000 | `formation.environments[0].service_speed_kn` | absent | 9.7 | changed |
| calibration amaggi-box-2000 | `formation.equivalent_hull.beam_m` | absent | 10.67 | changed |
| calibration amaggi-box-2000 | `formation.equivalent_hull.block_coefficient` | absent | 0.85831807902 | changed |
| calibration amaggi-box-2000 | `formation.equivalent_hull.draught_m` | absent | 3.56 | changed |
| calibration amaggi-box-2000 | `formation.equivalent_hull.length_m` | absent | 67.3018471817 | changed |
| calibration amaggi-box-2000 | `formation.method_status` | absent | unverified | changed |
| calibration amaggi-box-2000 | `formation.notes[0]` | absent | The Formation's resistance is that of its Equivalent Hull (the barges and the pusher as one hull) by the single-hull method, times the Formation resistance allowance. The method is unverified. | changed |
| calibration amaggi-box-2000 | `formation.notes[1]` | absent | Coupling force between the pusher and the pushed craft: 99 kN (ES-TRIN 2025/1, ESI-III-6 2.1: 270 x P_B x L_S / B_S, with P_B 259 kW, L_S 6.3 m taken as the pusher length, B_S 4.5 m). | changed |
| calibration amaggi-box-2000 | `formation.resistance_allowance` | absent | 1 | changed |
| calibration amaggi-box-2000 | `formation.sizing.converged` | absent | True | changed |
| calibration amaggi-box-2000 | `formation.sizing.max_passes` | absent | 5 | changed |
| calibration amaggi-box-2000 | `formation.sizing.passes` | absent | 3 | changed |
| calibration amaggi-box-2000 | `formation.sizing.tolerance` | absent | 0.01 | changed |
| calibration amaggi-box-2000 | `formation.tug.beam_m` | absent | 4.46770086056 | changed |
| calibration amaggi-box-2000 | `formation.tug.bollard_pull_t` | absent | 8.08533603776 | changed |
| calibration amaggi-box-2000 | `formation.tug.depth_m` | absent | 3 | changed |
| calibration amaggi-box-2000 | `formation.tug.draught_m` | absent | 2 | changed |
| calibration amaggi-box-2000 | `formation.tug.hull_steel_t` | absent | 8.50004283343 | changed |
| calibration amaggi-box-2000 | `formation.tug.installed_power_kw` | absent | 259.387339661 | changed |
| calibration amaggi-box-2000 | `formation.tug.installed_power_sized` | absent | True | changed |
| calibration amaggi-box-2000 | `formation.tug.length_m` | absent | 6.34184718175 | changed |
| calibration amaggi-box-2000 | `formation.tug.lightship_t` | absent | 30.0232053222 | changed |
| calibration amaggi-box-2000 | `formation.tug.machinery_t` | absent | 21.5231624888 | changed |
| calibration amaggi-box-2000 | `formation.tug.propeller_diameter_m` | absent | 1.6 | changed |
| calibration amaggi-box-2000 | `formation.tug.propulsor_count` | absent | 2 | changed |
| calibration amaggi-box-2000 | `formation.tug.propulsor_type` | absent | shaft_nozzle | changed |
| calibration amaggi-box-2000 | `formation.tug_included` | absent | True | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.coupling_force_kn` | absent | 62.8975612704 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.design_speed_kn` | absent | 9.7 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.environments[0].name` | absent | Deep water | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.environments[0].power_at_service_kw` | absent | 148.553198483 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.environments[0].service_speed_kn` | absent | 9.7 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.equivalent_hull.beam_m` | absent | 7.62 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.equivalent_hull.block_coefficient` | absent | 0.888731283183 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.equivalent_hull.draught_m` | absent | 2.7432 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.equivalent_hull.length_m` | absent | 65.8895597403 | changed |
| ... | 3097 more changed quantities | | | |
