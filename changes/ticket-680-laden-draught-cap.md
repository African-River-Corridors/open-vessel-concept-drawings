<!-- unreleased -->
- The draught cap binds the draught a dry bulk hull reports (issue #680). With its stern block mass a dry bulk hull floats trimmed, and at the displacement of the level cap its mid-length draught could sit up to a few centimetres below the cap, so its freeboard was less than `min_freeboard_m` (76.5 m bauxite hull at 0.8 m: draught 2.737 m against a 2.70 m cap, freeboard 0.763 m). The cargo is now the most whose reported draught is at the cap, whichever cap governs (hull, draught limit or channel). Five sweep cases move: cargo 0.16 % to 0.81 % less, draught 5 mm to 14 mm less, and each draught now sits on its cap.

<!-- design-impact -->
### ticket/680-laden-draught-cap (2026-10-30)

Between `main c0446ef` and `this branch 8b298a15`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep CN-derived bulk 2 (130x16.3) | `checks[14].message` | intact GM 3.80 m >= 0.15 m at the reported cargo 5956 t, draught 4.30 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 3.81 m >= 0.15 m at the reported cargo 5941 t, draught 4.29 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[16].message` | GM 3.78 m (fluid), max GZ 0.83 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | GM 3.79 m (fluid), max GZ 0.84 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| sweep CN-derived bulk 2 (130x16.3) | `checks[17].message` | GM 3.78 m (fluid), max GZ 0.83 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | GM 3.79 m (fluid), max GZ 0.83 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[14].message` | intact GM 4.49 m >= 0.15 m at the reported cargo 6333 t, draught 4.30 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 4.49 m >= 0.15 m at the reported cargo 6323 t, draught 4.30 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[16].message` | GM 4.46 m (fluid), max GZ 0.92 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | GM 4.47 m (fluid), max GZ 0.93 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| sweep CN-derived bulk 3 (130x17.2) | `checks[17].message` | GM 4.46 m (fluid), max GZ 0.92 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | GM 4.47 m (fluid), max GZ 0.93 m at 22° — fails: Angle of max GZ 22.000 < 25.0 | changed |
| sweep hull midship section in a channel 80x11.4 | `checks[14].message` | intact GM 3.24 m >= 0.15 m at the reported cargo 1488 t, draught 2.50 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 3.26 m >= 0.15 m at the reported cargo 1476 t, draught 2.49 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| sweep hull midship section in a channel 80x11.4 | `checks[15].message` | Channel: service 4.0 kn <= 0.95 x Schijf limit 4.47 kn | Channel: service 4.0 kn <= 0.95 x Schijf limit 4.48 kn | changed |
| sweep hull midship section in a channel 80x11.4 | `checks[16].message` | Channel: blockage n = 0.244 must be in (0, 1) | Channel: blockage n = 0.242 must be in (0, 1) | changed |
| sweep hull midship section in a channel 80x11.4 | `checks[17].message` | dynamic UKC 0.23 m = static 0.49 m - squat 0.26 m (Barrass II) at 4.00 kn | dynamic UKC 0.24 m = static 0.50 m - squat 0.26 m (Barrass II) at 4.00 kn | changed |
| sweep hull midship section in a channel 80x11.4 | `checks[18].message` | dynamic UKC 0.23 m against a 0.15 m working margin | dynamic UKC 0.24 m against a 0.15 m working margin | changed |
| sweep hull midship section in a channel 80x11.4 | `checks[20].message` | blockage 0.244; Barrass is fitted below about 0.35 and extrapolates above it | blockage 0.242; Barrass is fitted below about 0.35 and extrapolates above it | changed |
| sweep hull midship section in a channel 80x11.4 | `checks[24].message` | GM 3.19 m (fluid), max GZ 1.05 m at 26° | GM 3.21 m (fluid), max GZ 1.07 m at 26° | changed |
| sweep hull midship section in a channel 80x11.4 | `checks[25].message` | GM 3.20 m (fluid), max GZ 1.04 m at 26° | GM 3.22 m (fluid), max GZ 1.06 m at 26° | changed |
| sweep optional vessel_class | `checks[15].message` | intact GM 2.30 m >= 0.15 m at the reported cargo 2280 t, draught 2.80 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 2.31 m >= 0.15 m at the reported cargo 2264 t, draught 2.79 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| sweep optional vessel_class | `checks[17].message` | GM 2.28 m (fluid), max GZ 0.93 m at 29° | GM 2.29 m (fluid), max GZ 0.94 m at 29° | changed |
| sweep optional vessel_class | `checks[18].message` | GM 2.27 m (fluid), max GZ 0.93 m at 29° | GM 2.29 m (fluid), max GZ 0.94 m at 29° | changed |
| sweep optional vessel_scheme | `checks[15].message` | intact GM 2.58 m >= 0.15 m at the reported cargo 2422 t, draught 2.80 m (first-order; cargo KG of the full hold; KG_light @0.5D) | intact GM 2.60 m >= 0.15 m at the reported cargo 2406 t, draught 2.79 m (first-order; cargo KG of the full hold; KG_light @0.5D) | changed |
| sweep optional vessel_scheme | `checks[17].message` | GM 2.55 m (fluid), max GZ 1.01 m at 29° | GM 2.57 m (fluid), max GZ 1.03 m at 29° | changed |
| sweep optional vessel_scheme | `checks[18].message` | GM 2.55 m (fluid), max GZ 1.01 m at 29° | GM 2.57 m (fluid), max GZ 1.02 m at 29° | changed |
| sweep CN-derived bulk 2 (130x16.3) | `loading_conditions.conditions[1].gz[56][1]` | 0.00755512662723 | 0.0141691054871 | +87.5 % |
| sweep CN-derived bulk 2 (130x16.3) | `stability.conditions[1].gz[56][1]` | 0.00755512662723 | 0.0141691054871 | +87.5 % |
| sweep CN-derived bulk 2 (130x16.3) | `loading_conditions.conditions[0].gz[56][1]` | 0.0139170081043 | 0.0205422516975 | +47.6 % |
| sweep CN-derived bulk 2 (130x16.3) | `stability.conditions[0].gz[56][1]` | 0.0139170081043 | 0.0205422516975 | +47.6 % |
| sweep CN-derived bulk 3 (130x17.2) | `loading_conditions.conditions[1].gz[58][1]` | 0.00990580070599 | 0.0143666646264 | +45 % |
| sweep CN-derived bulk 3 (130x17.2) | `stability.conditions[1].gz[58][1]` | 0.00990580070599 | 0.0143666646264 | +45 % |
| sweep CN-derived bulk 2 (130x16.3) | `loading_conditions.conditions[0].gz[57][1]` | -0.020605547047 | -0.0141157465983 | +31.5 % |
| sweep CN-derived bulk 2 (130x16.3) | `stability.conditions[0].gz[57][1]` | -0.020605547047 | -0.0141157465983 | +31.5 % |
| sweep CN-derived bulk 3 (130x17.2) | `loading_conditions.conditions[0].gz[58][1]` | 0.016312250089 | 0.0207781605855 | +27.4 % |
| sweep CN-derived bulk 3 (130x17.2) | `stability.conditions[0].gz[58][1]` | 0.016312250089 | 0.0207781605855 | +27.4 % |
| sweep CN-derived bulk 2 (130x16.3) | `loading_conditions.conditions[1].gz[57][1]` | -0.0270454921323 | -0.0205671812516 | +24 % |
| sweep CN-derived bulk 2 (130x16.3) | `stability.conditions[1].gz[57][1]` | -0.0270454921323 | -0.0205671812516 | +24 % |
| sweep CN-derived bulk 3 (130x17.2) | `loading_conditions.conditions[0].gz[59][1]` | -0.0190954299044 | -0.014732810639 | +22.8 % |
| sweep CN-derived bulk 3 (130x17.2) | `stability.conditions[0].gz[59][1]` | -0.0190954299044 | -0.014732810639 | +22.8 % |
| sweep CN-derived bulk 3 (130x17.2) | `loading_conditions.conditions[1].gz[59][1]` | -0.0255659216254 | -0.0212085872571 | +17 % |
| sweep CN-derived bulk 3 (130x17.2) | `stability.conditions[1].gz[59][1]` | -0.0255659216254 | -0.0212085872571 | +17 % |
| sweep CN-derived bulk 2 (130x16.3) | `loading_conditions.conditions[1].gz[55][1]` | 0.0419894983221 | 0.0487360280749 | +16.1 % |
| sweep CN-derived bulk 2 (130x16.3) | `stability.conditions[1].gz[55][1]` | 0.0419894983221 | 0.0487360280749 | +16.1 % |
| sweep CN-derived bulk 2 (130x16.3) | `loading_conditions.conditions[0].gz[55][1]` | 0.0482703905943 | 0.0550279475037 | +14 % |
| sweep CN-derived bulk 2 (130x16.3) | `stability.conditions[0].gz[55][1]` | 0.0482703905943 | 0.0550279475037 | +14 % |
| ... | 1931 more changed quantities | | | |
