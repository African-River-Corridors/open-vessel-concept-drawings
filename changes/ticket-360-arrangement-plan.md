<!-- unreleased -->
- The pusher sees over its convoy (issue #360, part 1): with the tug's sightline method `blind_distance`, the result of a pushed barge, a pushed convoy or an ATB with its tug has `formation.sightline`. It gives the Blind Distance light and laden over every barge ahead (each barge's highest point at its forward end), the wheelhouse lift the ES-TRIN 7.02(2) limit of 250 m needs, and whether the lift is enough. The eye is at the forward face of the wheelhouse, at mid-length of the tug, with the floor on its deck (estimates). A tug that cannot see far enough gets the warning `warn-formation-blind-distance-lift`. A file with `margin` (the stored default) is as it was.
- New drawing `arrangement-plan` (issue #360, part 1): one sheet for a pushed convoy, or a pushed barge or an ATB with its tug. The plan shows the barges and the tug, the overall length and beam, the barge length and the coupling points. The profile below shows the light condition and the tug's sightline over the convoy, with the wheelhouse raised. `ovcd draw` writes it beside the GA views; a pushed convoy gets only this sheet in place of the message "no drawings of a pushed convoy yet". The pusher's own GA is not drawn yet.

<!-- design-impact -->
### ticket/360-arrangement-plan (2026-10-23)

Between `main 9fc7e39` and `this branch fb0696dc`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `formation.sightline.blind_distance_laden_m` | absent | 26.1838360534 | changed |
| calibration amaggi-box-2000 | `formation.sightline.blind_distance_light_m` | absent | 249.999999999 | changed |
| calibration amaggi-box-2000 | `formation.sightline.bow_x_m` | absent | 70.4299869491 | changed |
| calibration amaggi-box-2000 | `formation.sightline.enough` | absent | True | changed |
| calibration amaggi-box-2000 | `formation.sightline.eye_lowered_above_water_m` | absent | 2.5 | changed |
| calibration amaggi-box-2000 | `formation.sightline.eye_raised_above_water_m` | absent | 6.00037359585 | changed |
| calibration amaggi-box-2000 | `formation.sightline.eye_x_m` | absent | 4.73499347453 | changed |
| calibration amaggi-box-2000 | `formation.sightline.floor_raised_above_water_m` | absent | 4.50037359585 | changed |
| calibration amaggi-box-2000 | `formation.sightline.lift_needed_m` | absent | 3.50037359584 | changed |
| calibration amaggi-box-2000 | `formation.sightline.lift_used_m` | absent | 3.50037359585 | changed |
| calibration amaggi-box-2000 | `formation.sightline.limit_m` | absent | 250 | changed |
| calibration amaggi-box-2000 | `formation.sightline.max_lift_m` | absent | 6 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.blind_distance_laden_m` | absent | 36.4053494856 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.blind_distance_light_m` | absent | 250 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.bow_x_m` | absent | 68.2531043046 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.enough` | absent | True | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.eye_lowered_above_water_m` | absent | 2.5 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.eye_raised_above_water_m` | absent | 5.31177937385 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.eye_x_m` | absent | 3.64655215229 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.floor_raised_above_water_m` | absent | 3.81177937385 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.lift_needed_m` | absent | 2.81177937385 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.lift_used_m` | absent | 2.81177937385 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.limit_m` | absent | 250 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.max_lift_m` | absent | 6 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.blind_distance_laden_m` | absent | 36.6716508473 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.blind_distance_light_m` | absent | 250 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.bow_x_m` | absent | 69.6501065412 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.enough` | absent | True | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.eye_lowered_above_water_m` | absent | 2.5 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.eye_raised_above_water_m` | absent | 5.32357278314 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.eye_x_m` | absent | 4.34505327061 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.floor_raised_above_water_m` | absent | 3.82357278314 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.lift_needed_m` | absent | 2.82357278314 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.lift_used_m` | absent | 2.82357278314 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.limit_m` | absent | 250 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.max_lift_m` | absent | 6 | changed |
| calibration canal-box-200x35x12-imt201 | `formation.sightline.blind_distance_laden_m` | absent | 36.6716508473 | changed |
| calibration canal-box-200x35x12-imt201 | `formation.sightline.blind_distance_light_m` | absent | 250 | changed |
| calibration canal-box-200x35x12-imt201 | `formation.sightline.bow_x_m` | absent | 69.6501065412 | changed |
| calibration canal-box-200x35x12-imt201 | `formation.sightline.enough` | absent | True | changed |
| ... | 1182 more changed quantities | | | |
