<!-- unreleased -->
- Design change (issue #526): resistance, squat, waves and the bollard pull take one value of g (9.80665 m/s^2) and one knot (1852/3600 m/s) from one definition (`constants.rs`). Before, the resistance used 9.81 and 0.5144, and squat and waves used 9.80665 and 0.514444, so one speed point mixed both pairs. The speed to metres per second factor moves by 0.01 % and g by 0.03 %: small shifts in resistance, power, squat, wave resistance and bollard pull on every Design with a speed curve. The Python API (`squat`, `waves.compute`) differs from 0.1.0 in the same way and is listed in `KNOWN_DIFFERENCES`.
- Examples (issue #409): the dry bulk box hold and the two gas example files are saved in the IS Code loading conditions, so their drawings show the ballast waterline and the ballast tanks. The dry bulk example now has a stability result. All stored results hashes are re-stamped.

<!-- design-impact -->
### ticket/526-one-g-knot (2026-10-22)

Between `main ab89a78` and `this branch 5e9ad0e`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep crude channel 90x14 | `checks[5].message` | Channel: service 7.5 kn <= 0.95 x Schijf limit 8.07 kn | Channel: service 7.5 kn <= 0.95 x Schijf limit 8.06 kn | changed |
| sweep hopper channel 60x10 | `checks[15].message` | Channel: service 9.0 kn <= 0.95 x Schijf limit 9.69 kn | Channel: service 9.0 kn <= 0.95 x Schijf limit 9.68 kn | changed |
| sweep optional channel_under_keel_clearance_m | `checks[6].message` | Channel: service 4.0 kn <= 0.95 x Schijf limit 4.25 kn | Channel: service 4.0 kn <= 0.95 x Schijf limit 4.24 kn | changed |
| sweep CN-derived liquid 2 (130x16.3) | `loading_conditions.conditions[0].gz[57][1]` | -8.211692023e-05 | -8.30334010178e-05 | -1.12 % |
| sweep CN-derived liquid 2 (130x16.3) | `stability.conditions[0].gz[57][1]` | -8.211692023e-05 | -8.30334010178e-05 | -1.12 % |
| sweep box self-propelled 60x10 | `loading_conditions.conditions[0].gz[58][1]` | -0.000988771695421 | -0.000984665289963 | +0.415 % |
| sweep box self-propelled 60x10 | `stability.conditions[0].gz[58][1]` | -0.000988771695421 | -0.000984665289963 | +0.415 % |
| sweep gas IGC_2G 60x10 | `loading_conditions.conditions[1].gz[59][1]` | 0.000803167014991 | 0.000802304002826 | -0.107 % |
| sweep gas IGC_2G 60x10 | `stability.conditions[1].gz[59][1]` | 0.000803167014991 | 0.000802304002826 | -0.107 % |
| sweep gas IGC_2G_EXACT 60x10 | `loading_conditions.conditions[1].gz[59][1]` | 0.000803167014991 | 0.000802304002826 | -0.107 % |
| sweep gas IGC_2G_EXACT 60x10 | `stability.conditions[1].gz[59][1]` | 0.000803167014991 | 0.000802304002826 | -0.107 % |
| sweep hopper 90x14 | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep hopper 90x14 | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional accommodation_unit_count_dry | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional accommodation_unit_count_dry | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional battery_unit_count_dry | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional battery_unit_count_dry | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional bow_rake_length_m | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional bow_rake_length_m | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional bow_rake_length_to_depth_ratio | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional bow_rake_length_to_depth_ratio | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional bow_rake_rise_m | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional bow_rake_rise_m | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional chine_half_breadth_m | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional chine_half_breadth_m | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional chine_height_m | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional chine_height_m | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional deadrise_deg | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional deadrise_deg | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional hatch_coaming_plate_mm | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional hatch_coaming_plate_mm | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional hold_bulkhead_plate_mm | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional hold_bulkhead_plate_mm | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional hopper_plate_mm | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional hopper_plate_mm | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional hopper_slope_h_per_v | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional hopper_slope_h_per_v | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional hull_offsets | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional hull_offsets | `stability.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| sweep optional inner_bottom_plate_mm | `loading_conditions.conditions[1].gz[57][1]` | 0.00114481196123 | 0.00114364597197 | -0.102 % |
| ... | 108238 more changed quantities | | | |
