<!-- unreleased -->
- The pusher sees over its convoy (issue #360, part 1): with the tug's sightline method `blind_distance`, the result of a pushed barge, a pushed convoy or an ATB with its tug has `formation.sightline`. It gives the Blind Distance light and laden over every barge ahead (each barge's highest point at its forward end), the wheelhouse lift the ES-TRIN 7.02(2) limit of 250 m needs, and whether the lift is enough. The eye is at the forward face of the wheelhouse, at mid-length of the tug, with the floor on its deck (estimates). A tug that cannot see far enough gets the warning `warn-formation-blind-distance-lift`. A file with `margin` (the stored default) is as it was.
- New drawing `arrangement-plan` (issue #360, part 1): one sheet for a pushed convoy, or a pushed barge or an ATB with its tug. The plan shows the barges and the tug, the overall length and beam, the barge length and the coupling points. The profile below shows the light condition and the tug's sightline over the convoy, with the wheelhouse raised. `ovcd draw` writes it beside the GA views; a pushed convoy gets only this sheet in place of the message "no drawings of a pushed convoy yet". The pusher's own GA is not drawn yet.

<!-- design-impact -->
### ticket/360-arrangement-plan (2026-10-23)

Between `main de050d9` and `this branch 7836b5f6`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| calibration amaggi-box-2000 | `formation.sightline.blind_distance_laden_m` | absent | 25.7389542317 | changed |
| calibration amaggi-box-2000 | `formation.sightline.blind_distance_light_m` | absent | 250 | changed |
| calibration amaggi-box-2000 | `formation.sightline.bow_x_m` | absent | 67.3029625057 | changed |
| calibration amaggi-box-2000 | `formation.sightline.enough` | absent | True | changed |
| calibration amaggi-box-2000 | `formation.sightline.eye_lowered_above_water_m` | absent | 2.5 | changed |
| calibration amaggi-box-2000 | `formation.sightline.eye_raised_above_water_m` | absent | 5.97065612282 | changed |
| calibration amaggi-box-2000 | `formation.sightline.eye_x_m` | absent | 3.17148125284 | changed |
| calibration amaggi-box-2000 | `formation.sightline.floor_raised_above_water_m` | absent | 4.47065612282 | changed |
| calibration amaggi-box-2000 | `formation.sightline.lift_needed_m` | absent | 3.47065612282 | changed |
| calibration amaggi-box-2000 | `formation.sightline.lift_used_m` | absent | 3.47065612282 | changed |
| calibration amaggi-box-2000 | `formation.sightline.limit_m` | absent | 250 | changed |
| calibration amaggi-box-2000 | `formation.sightline.max_lift_m` | absent | 6 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.blind_distance_laden_m` | absent | 35.9507137069 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.blind_distance_light_m` | absent | 249.999999999 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.bow_x_m` | absent | 65.89031479 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.enough` | absent | True | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.eye_lowered_above_water_m` | absent | 2.5 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.eye_raised_above_water_m` | absent | 5.29183284617 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.eye_x_m` | absent | 2.46515739501 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.floor_raised_above_water_m` | absent | 3.79183284617 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.lift_needed_m` | absent | 2.79183284616 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.lift_used_m` | absent | 2.79183284617 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.limit_m` | absent | 250 | changed |
| calibration canal-box-200x25x12-pb2230 | `formation.sightline.max_lift_m` | absent | 6 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.blind_distance_laden_m` | absent | 36.1288467893 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.blind_distance_light_m` | absent | 250 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.bow_x_m` | absent | 66.8127776462 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.enough` | absent | True | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.eye_lowered_above_water_m` | absent | 2.5 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.eye_raised_above_water_m` | absent | 5.29962022239 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.eye_x_m` | absent | 2.9263888231 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.floor_raised_above_water_m` | absent | 3.79962022239 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.lift_needed_m` | absent | 2.79962022239 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.lift_used_m` | absent | 2.79962022239 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.limit_m` | absent | 250 | changed |
| calibration canal-box-200x35x12-cby222 | `formation.sightline.max_lift_m` | absent | 6 | changed |
| calibration canal-box-200x35x12-imt201 | `formation.sightline.blind_distance_laden_m` | absent | 36.1288467893 | changed |
| calibration canal-box-200x35x12-imt201 | `formation.sightline.blind_distance_light_m` | absent | 250 | changed |
| calibration canal-box-200x35x12-imt201 | `formation.sightline.bow_x_m` | absent | 66.8127776462 | changed |
| calibration canal-box-200x35x12-imt201 | `formation.sightline.enough` | absent | True | changed |
| ... | 1182 more changed quantities | | | |
