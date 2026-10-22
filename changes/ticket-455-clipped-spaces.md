<!-- unreleased -->
- Design change (issue #455): Spaces are clipped by the hull (`compartments::clip`, spec 0008 "One compartment model"). Each hull section is clipped by the Space's convex section and the result is integrated along x by Simpson's rule, split where the shape changes. A Space wholly inside the hull keeps its prism values bit for bit. The fore peak tank is now the whole bow void, clipped by the bow rake: with a rake that rises the full depth its volume is the same as before, but its centroid and its fills follow the rake (water in the rake sits higher and further aft). Bilge radii and rakes that cut the double-bottom, wing and aft peak tanks give their true volumes. The ballast capacities and centroids in `loading_conditions` move, and so do the ballast conditions that use them. The Vessel Model's Spaces are clipped too; a dry bulk hold loses its coaming part, which lies above the deck. No check changes its verdict on the design-impact cases.

<!-- design-impact -->
### ticket/455-clipped-spaces (2026-10-22)

Between `main 54e0b1f` and `this branch 8c69483`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep gas cargo LNG pushed | `checks[16].message` | GM 5.74 m (fluid), max GZ 1.88 m at 25° | GM 4.60 m (fluid), max GZ 1.62 m at 25° | changed |
| sweep gas cargo LNG pushed | `checks[17].message` | GM 5.74 m (fluid), max GZ 1.88 m at 25° | GM 4.60 m (fluid), max GZ 1.62 m at 25° | changed |
| sweep gas cargo LNG pushed | `checks[18].message` | GM 9.52 m (fluid), max GZ 2.34 m at 21° — warning, not a failure: angle of max GZ 21 < 25 (IS Code 2.2.3) with no ballast; every other criterion holds. To sail light, take 537 t of ballast (the Ballast, departure condition meets every criterion) | GM 9.52 m (fluid), max GZ 2.34 m at 21° — warning, not a failure: angle of max GZ 21 < 25 (IS Code 2.2.3) with no ballast; every other criterion holds. To sail light, take 708 t of ballast (the Ballast, departure condition meets every criterion) | changed |
| sweep gas cargo LNG_ethane pushed | `checks[16].message` | GM 5.68 m (fluid), max GZ 1.88 m at 25° | GM 4.74 m (fluid), max GZ 1.64 m at 25° | changed |
| sweep gas cargo LNG_ethane pushed | `checks[17].message` | GM 5.68 m (fluid), max GZ 1.88 m at 25° | GM 4.74 m (fluid), max GZ 1.64 m at 25° | changed |
| sweep gas cargo LNG_ethane pushed | `checks[18].message` | GM 9.42 m (fluid), max GZ 2.33 m at 21° — warning, not a failure: angle of max GZ 21 < 25 (IS Code 2.2.3) with no ballast; every other criterion holds. To sail light, take 537 t of ballast (the Ballast, departure condition meets every criterion) | GM 9.42 m (fluid), max GZ 2.33 m at 21° — warning, not a failure: angle of max GZ 21 < 25 (IS Code 2.2.3) with no ballast; every other criterion holds. To sail light, take 665 t of ballast (the Ballast, departure condition meets every criterion) | changed |
| sweep gas channel 60x10 | `checks[16].message` | GM 4.35 m (fluid), max GZ 1.38 m at 25° | GM 3.30 m (fluid), max GZ 1.11 m at 25° | changed |
| sweep gas channel 60x10 | `checks[17].message` | GM 4.35 m (fluid), max GZ 1.38 m at 25° | GM 3.30 m (fluid), max GZ 1.11 m at 25° | changed |
| sweep gas channel 60x10 | `checks[18].message` | GM 8.76 m (fluid), max GZ 1.96 m at 21° — warning, not a failure: angle of max GZ 21 < 25 (IS Code 2.2.3) with no ballast; every other criterion holds. To sail light, take 241 t of ballast (the Ballast, departure condition meets every criterion) | GM 8.76 m (fluid), max GZ 1.96 m at 21° — warning, not a failure: angle of max GZ 21 < 25 (IS Code 2.2.3) with no ballast; every other criterion holds. To sail light, take 310 t of ballast (the Ballast, departure condition meets every criterion) | changed |
| sweep gas channel 90x14 | `checks[16].message` | GM 5.92 m (fluid), max GZ 1.92 m at 25° | GM 4.78 m (fluid), max GZ 1.64 m at 25° | changed |
| sweep gas channel 90x14 | `checks[17].message` | GM 5.92 m (fluid), max GZ 1.92 m at 25° | GM 4.78 m (fluid), max GZ 1.64 m at 25° | changed |
| sweep gas channel 90x14 | `checks[18].message` | GM 9.92 m (fluid), max GZ 2.38 m at 21° — warning, not a failure: angle of max GZ 21 < 25 (IS Code 2.2.3) with no ballast; every other criterion holds. To sail light, take 552 t of ballast (the Ballast, departure condition meets every criterion) | GM 9.92 m (fluid), max GZ 2.38 m at 21° — warning, not a failure: angle of max GZ 21 < 25 (IS Code 2.2.3) with no ballast; every other criterion holds. To sail light, take 713 t of ballast (the Ballast, departure condition meets every criterion) | changed |
| sweep hull offsets 76.5x11.4 | `checks[10].message` | GM 7.87 m (fluid), max GZ 2.10 m at 25° | GM 7.89 m (fluid), max GZ 2.10 m at 25° | changed |
| sweep hull offsets 76.5x11.4 | `checks[9].message` | GM 7.87 m (fluid), max GZ 2.10 m at 25° | GM 7.89 m (fluid), max GZ 2.10 m at 25° | changed |
| sweep optional ballast_plan | `checks[9].message` | GM 8.57 m (fluid), max GZ 2.35 m at 24° — fails: Angle of max GZ 24.000 < 25.0 | GM 8.45 m (fluid), max GZ 2.31 m at 23° — fails: Angle of max GZ 23.000 < 25.0 | changed |
| sweep optional vessel_class | `checks[19].message` | GM 4.43 m (fluid), max GZ 2.12 m at 37° | GM 4.37 m (fluid), max GZ 2.08 m at 36° | changed |
| sweep optional vessel_class | `checks[22].message` | GM 4.45 m (fluid), max GZ 2.11 m at 37° | GM 4.39 m (fluid), max GZ 2.07 m at 36° | changed |
| sweep optional vessel_scheme | `checks[19].message` | GM 4.90 m (fluid), max GZ 2.26 m at 36° | GM 4.84 m (fluid), max GZ 2.22 m at 36° | changed |
| sweep optional vessel_scheme | `checks[22].message` | GM 4.92 m (fluid), max GZ 2.25 m at 36° | GM 4.86 m (fluid), max GZ 2.21 m at 36° | changed |
| sweep gas channel 60x10 | `loading_conditions.conditions[2].gz[60][1]` | 0.168113603517 | -0.250941565353 | -249 % |
| sweep gas channel 60x10 | `loading_conditions.conditions[3].gz[60][1]` | 0.168113603517 | -0.250941565353 | -249 % |
| sweep gas channel 60x10 | `stability.conditions[2].gz[60][1]` | 0.168113603517 | -0.250941565353 | -249 % |
| sweep gas channel 60x10 | `stability.conditions[3].gz[60][1]` | 0.168113603517 | -0.250941565353 | -249 % |
| sweep gas channel 60x10 | `loading_conditions.conditions[2].gz[59][1]` | 0.224266750108 | -0.192253538973 | -186 % |
| sweep gas channel 60x10 | `loading_conditions.conditions[3].gz[59][1]` | 0.224266750108 | -0.192253538973 | -186 % |
| sweep gas channel 60x10 | `stability.conditions[2].gz[59][1]` | 0.224266750108 | -0.192253538973 | -186 % |
| sweep gas channel 60x10 | `stability.conditions[3].gz[59][1]` | 0.224266750108 | -0.192253538973 | -186 % |
| sweep gas channel 90x14 | `loading_conditions.conditions[2].gz[60][1]` | 0.26193543596 | -0.159957719097 | -161 % |
| sweep gas channel 90x14 | `loading_conditions.conditions[3].gz[60][1]` | 0.26193543596 | -0.159957719097 | -161 % |
| sweep gas channel 90x14 | `stability.conditions[2].gz[60][1]` | 0.26193543596 | -0.159957719097 | -161 % |
| sweep gas channel 90x14 | `stability.conditions[3].gz[60][1]` | 0.26193543596 | -0.159957719097 | -161 % |
| sweep gas channel 60x10 | `loading_conditions.conditions[2].gz[58][1]` | 0.279934618547 | -0.133882399651 | -148 % |
| sweep gas channel 60x10 | `loading_conditions.conditions[3].gz[58][1]` | 0.279934618547 | -0.133882399651 | -148 % |
| sweep gas channel 60x10 | `stability.conditions[2].gz[58][1]` | 0.279934618547 | -0.133882399651 | -148 % |
| sweep gas channel 60x10 | `stability.conditions[3].gz[58][1]` | 0.279934618547 | -0.133882399651 | -148 % |
| sweep gas cargo LNG pushed | `loading_conditions.conditions[2].gz[60][1]` | 0.306991545658 | -0.0846694905296 | -128 % |
| sweep gas cargo LNG pushed | `loading_conditions.conditions[3].gz[60][1]` | 0.306991545658 | -0.0846694905296 | -128 % |
| sweep gas cargo LNG pushed | `stability.conditions[2].gz[60][1]` | 0.306991545658 | -0.0846694905296 | -128 % |
| sweep gas cargo LNG pushed | `stability.conditions[3].gz[60][1]` | 0.306991545658 | -0.0846694905296 | -128 % |
| sweep gas cargo LNG_ethane pushed | `loading_conditions.conditions[2].gz[60][1]` | 0.298784061272 | -0.0698614353302 | -123 % |
| ... | 3205 more changed quantities | | | |
