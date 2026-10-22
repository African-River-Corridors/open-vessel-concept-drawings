<!-- unreleased -->
- Formations (issue #362, spec 0007, part 2 of the ATB: the powering): an `atb` Formation that includes its tug (`tug_included`) gets the `formation` block of a pushed one, with the tug sized from the barge (0.28 of its length, 0.53 of its beam, draught beam over 2.33; typed values kept) and the Equivalent Hull of the barge and the tug less the Notch the tug sits in (length barge + tug - notch depth, the barge's beam and draught, the block coefficient of the displaced volumes over L x B x T). The Formation's resistance is that hull's by the single-hull method times the Formation resistance allowance (1.05 for an ATB); the power and the service speed come in each environment from the tug's propulsor, with the tug's own lightship in the displacement. The block holds the Notch depth and has no coupling force (a pusher's formula; the ATB's linkage loads are approval items). The method is `unverified`. A pushed Formation's block is unchanged. Calibration: `calibration/atb.yaml` and `atb_powering.rs` compare the model with the Aveogan, the 550 Class and the Kim M. Bouchard: the still-water deep-water power the model needs at the service speed is 0.29 to 0.43 of the published installed power (which also covers shallow water, seaway and bollard pull; the barge depths are estimates).

<!-- design-impact -->
### ticket/362-atb-powering (2026-10-22)

Between `main 263faba` and `this branch 7476161`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep atb notch 100x20 | `formation.design_speed_kn` | absent | 9.7 | changed |
| sweep atb notch 100x20 | `formation.environments[0].name` | absent | Deep water | changed |
| sweep atb notch 100x20 | `formation.environments[0].power_at_service_kw` | absent | 580.247565277 | changed |
| sweep atb notch 100x20 | `formation.environments[0].service_speed_kn` | absent | 9.7 | changed |
| sweep atb notch 100x20 | `formation.equivalent_hull.beam_m` | absent | 20 | changed |
| sweep atb notch 100x20 | `formation.equivalent_hull.block_coefficient` | absent | 0.785483185914 | changed |
| sweep atb notch 100x20 | `formation.equivalent_hull.draught_m` | absent | 5.7 | changed |
| sweep atb notch 100x20 | `formation.equivalent_hull.length_m` | absent | 114 | changed |
| sweep atb notch 100x20 | `formation.method_status` | absent | unverified | changed |
| sweep atb notch 100x20 | `formation.notch_depth_m` | absent | 14 | changed |
| sweep atb notch 100x20 | `formation.notes[0]` | absent | The ATB's resistance is that of its Equivalent Hull (the barge and the tug as one hull, as long as the barge and the tug less the Notch the tug sits in) by the single-hull method, times the Formation resistance allowance. The method is unverified. | changed |
| sweep atb notch 100x20 | `formation.resistance_allowance` | absent | 1.05 | changed |
| sweep atb notch 100x20 | `formation.sizing.converged` | absent | True | changed |
| sweep atb notch 100x20 | `formation.sizing.max_passes` | absent | 5 | changed |
| sweep atb notch 100x20 | `formation.sizing.passes` | absent | 2 | changed |
| sweep atb notch 100x20 | `formation.sizing.tolerance` | absent | 0.01 | changed |
| sweep atb notch 100x20 | `formation.tug.beam_m` | absent | 10.6 | changed |
| sweep atb notch 100x20 | `formation.tug.bollard_pull_t` | absent | 22.881426297 | changed |
| sweep atb notch 100x20 | `formation.tug.depth_m` | absent | 5.54935622318 | changed |
| sweep atb notch 100x20 | `formation.tug.draught_m` | absent | 4.54935622318 | changed |
| sweep atb notch 100x20 | `formation.tug.hull_steel_t` | absent | 164.704892704 | changed |
| sweep atb notch 100x20 | `formation.tug.installed_power_kw` | absent | 696.297078333 | changed |
| sweep atb notch 100x20 | `formation.tug.installed_power_sized` | absent | True | changed |
| sweep atb notch 100x20 | `formation.tug.length_m` | absent | 28 | changed |
| sweep atb notch 100x20 | `formation.tug.lightship_t` | absent | 252.519484134 | changed |
| sweep atb notch 100x20 | `formation.tug.machinery_t` | absent | 87.8145914302 | changed |
| sweep atb notch 100x20 | `formation.tug.propeller_diameter_m` | absent | 2.95708154506 | changed |
| sweep atb notch 100x20 | `formation.tug.propulsor_count` | absent | 2 | changed |
| sweep atb notch 100x20 | `formation.tug.propulsor_type` | absent | azimuth_ducted | changed |
| sweep atb notch 100x20 | `formation.tug_included` | absent | True | changed |
