# ShipD benchmark

Written by `cargo run -p xtask -- shipd-benchmark` (issue #452, spec 0008 "Benchmark against the dataset"). The sample and its source are in `SOURCE.md`.

Each sampled hull is built at L = 10 m as OVCD's ShipD-style Sections Table (101 even stations plus the knuckles and the ends of each height's waterline; points at 200 even heights plus the knuckle heights) and measured upright at the dataset's ten draughts (T/Dd 0.12 to 1.0; at 1.0 a hair below the deck, where the closed section has its deck). A hull passes when, at every draught, its volume, waterplane area and waterline length are within 1 %, its LCB and LCF within 0.5 % of LOA, and its Ixx and Iyy within 2 % of the dataset's. The dataset integrates its waterlines by the trapezoid rule over strips (101 waterlines, 1000 points each) and interpolates between its waterlines; OVCD integrates its sections by Simpson's rule on the ruled surface between them: part of any difference is the dataset's. The tolerances are not loosened to make a hull pass; a hull outside is a finding, listed below with its numbers.

| Set | Hulls | Pass | Volume | Area_WP | LCB | LCF | Ixx | Iyy | WL_Length |
|---|---|---|---|---|---|---|---|---|---|
| Constrained_Randomized_Set_2 | 100 | 95 (95 %) | 97 | 100 | 100 | 100 | 98 | 100 | 100 |
| Constrained_Randomized_Set_3 | 100 | 98 (98 %) | 100 | 99 | 100 | 100 | 100 | 99 | 99 |

The last seven columns count the hulls within the limit for that quantity at every draught. The spec's pass is at least 95 % of each set.

## Hulls outside

| Set | Row | Outside the limit |
|---|---|---|
| Constrained_Randomized_Set_2 | 3251 | Volume at T/Dd 0.12: OVCD 0.001501, dataset 0.001468 (2.25 x the limit); |
| Constrained_Randomized_Set_2 | 2740 | Ixx at T/Dd 0.12: OVCD 0.000000, dataset 0.000000 (1.11 x the limit); |
| Constrained_Randomized_Set_2 | 4751 | Ixx at T/Dd 0.12: OVCD 0.000000, dataset 0.000000 (1.12 x the limit); |
| Constrained_Randomized_Set_2 | 7596 | Volume at T/Dd 0.12: OVCD 0.002764, dataset 0.002705 (2.18 x the limit); |
| Constrained_Randomized_Set_2 | 9672 | Volume at T/Dd 0.25: OVCD 0.001611, dataset 0.001635 (1.42 x the limit); |
| Constrained_Randomized_Set_3 | 5310 | WL_Length at T/Dd 1.00: OVCD 0.532639, dataset 0.523954 (1.66 x the limit); |
| Constrained_Randomized_Set_3 | 5710 | Area_WP at T/Dd 1.00: OVCD 0.089457, dataset 0.090558 (1.22 x the limit); Iyy at T/Dd 1.00: OVCD 0.001015, dataset 0.001051 (1.71 x the limit); |

## At a Design's cut

For information: a Design's ShipD hull is cut more coarsely (41 even stations and 20 even height levels, no waterline-end stations), so its hydrostatics and GZ curves stay fast. At that cut the same comparison gives:

| Set | Hulls | Pass | Volume | Area_WP | LCB | LCF | Ixx | Iyy | WL_Length |
|---|---|---|---|---|---|---|---|---|---|
| Constrained_Randomized_Set_2 | 100 | 0 (0 %) | 33 | 44 | 83 | 75 | 38 | 24 | 0 |
| Constrained_Randomized_Set_3 | 100 | 0 (0 %) | 16 | 71 | 97 | 82 | 69 | 39 | 0 |
