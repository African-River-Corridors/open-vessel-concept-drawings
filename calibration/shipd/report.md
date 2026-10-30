# ShipD benchmark

Written by `cargo run -p xtask -- shipd-benchmark` (issues #452, #453 and #648, spec 0008 "Benchmark against the dataset"). The sample and its source are in `SOURCE.md`.

The benchmark compares OVD's ShipD-style hull with the dataset's measures at its ten draughts (T/Dd 0.12 to 1.0), at L = 10 m. A hull passes when, at every draught, its volume, waterplane area and waterline length are within 1 %, its LCB and LCF within 0.5 % of LOA, and its Ixx and Iyy within 2 % of the dataset's. The tolerances are not loosened to make a hull pass.

## The gate: OVD's shape, measured as the dataset measures

The dataset took its measures from 101 waterlines with 1000 points each, by the trapezoid rule, and interpolated between waterlines. The gate measures OVD's shape the same way (`ovd_core::shipd::dataset_measures`), so it compares the shape, not the integration rule. Every hull must pass, except a hull listed in `artefacts.csv`: there the dataset's code departs from the definitions, and the gate requires that reading that one place as the code does brings the hull within the limits. Any other hull outside fails the gate.

| Set | Hulls | Pass | Dataset artefact | Volume | Area_WP | LCB | LCF | Ixx | Iyy | WL_Length |
|---|---|---|---|---|---|---|---|---|---|---|
| Constrained_Randomized_Set_2 | 100 | 100 | 0 | 100 | 100 | 100 | 100 | 100 | 100 | 100 |
| Constrained_Randomized_Set_3 | 100 | 100 | 0 | 100 | 100 | 100 | 100 | 100 | 100 | 100 |
| Constrained_Randomized_Set_1 | 100 | 94 | 6 | 97 | 97 | 100 | 97 | 97 | 97 | 100 |

The last seven columns count the hulls within the limit for that quantity at every draught.

## Hulls outside: the dataset's code

- `stern_ellipse`: The stern taper is a quarter ellipse (`bit_EP_T` or `bit_EP_S` = 1) and the transom is wider than the midbody at that height. The definitions run the ellipse from the midbody half-breadth to the transom's. The dataset's code squares its vertical radius (`Ry = y_mid - y_transom`, here negative) inside the root, so its ellipse is mirrored: it starts at `2 y_transom - y_mid`, a step out from the midbody, and falls to the transom.
- `bulb_crossing`: Where the bow bulb meets the hull, the dataset's code takes the smallest root of the bow cubic that lies aft of the stem and of the bulb's front. When the real crossing lies forward of the bulb's front, the root it takes is the real part of a complex root (numpy orders a complex number by its real part first) or a root aft of `delta_bow`. Its waterline then runs at the bulb's width to `delta_bow`: wider than the hull where the bulb is wider, or one waterline just below the bulb's top with up to 30 % less area than its neighbours.

| Set | Row | Cause | Outside, by the definitions | Read as the dataset's code |
|---|---|---|---|---|
| Constrained_Randomized_Set_1 | 9764 | `bulb_crossing` | Volume at T/Dd 0.12: OVD 0.001991, dataset 0.002032 (2.02 x the limit) | within the limits (worst 0.02 x the limit) |
| Constrained_Randomized_Set_1 | 7207 | `bulb_crossing` | Area_WP at T/Dd 0.38: OVD 0.180596, dataset 0.175834 (2.71 x the limit); LCF at T/Dd 0.38: OVD 0.449671, dataset 0.460095 (2.08 x the limit); Ixx at T/Dd 0.38: OVD 0.000749, dataset 0.000734 (1.01 x the limit); Iyy at T/Dd 0.38: OVD 0.009462, dataset 0.008737 (4.15 x the limit) | within the limits (worst 0.16 x the limit) |
| Constrained_Randomized_Set_1 | 5660 | `bulb_crossing` | Volume at T/Dd 0.25: OVD 0.006113, dataset 0.006048 (1.08 x the limit) | within the limits (worst 0.02 x the limit) |
| Constrained_Randomized_Set_1 | 5584 | `stern_ellipse` | Area_WP at T/Dd 1.00: OVD 0.163771, dataset 0.174939 (6.38 x the limit); LCF at T/Dd 1.00: OVD 0.462912, dataset 0.468370 (1.09 x the limit); Ixx at T/Dd 1.00: OVD 0.000528, dataset 0.000636 (8.48 x the limit); Iyy at T/Dd 1.00: OVD 0.009955, dataset 0.010312 (1.73 x the limit) | within the limits (worst 0.02 x the limit) |
| Constrained_Randomized_Set_1 | 5319 | `stern_ellipse` | Area_WP at T/Dd 1.00: OVD 0.196925, dataset 0.217625 (9.51 x the limit); LCF at T/Dd 1.00: OVD 0.492782, dataset 0.517583 (4.96 x the limit); Ixx at T/Dd 1.00: OVD 0.000683, dataset 0.000931 (13.32 x the limit); Iyy at T/Dd 1.00: OVD 0.017108, dataset 0.018647 (4.13 x the limit) | within the limits (worst 0.04 x the limit) |
| Constrained_Randomized_Set_1 | 1533 | `bulb_crossing` | Volume at T/Dd 0.25: OVD 0.005906, dataset 0.005824 (1.41 x the limit) | within the limits (worst 0.01 x the limit) |

## Hulls outside for any other reason

None.

## OVD's own integration

For information: the same comparison with OVD's own hydrostatics, its Sections Table cut at 101 even stations plus the knuckles and the ends of each height's waterline, and points at 200 even heights plus the knuckle heights, integrated by Simpson's rule on the ruled surface between sections (at T/Dd 1.0 a hair below the deck, where the closed section has its deck). The hulls outside here are mostly at low draught, where the dataset's trapezoid rule over its first waterlines differs from OVD's integral.

| Set | Hulls | Pass | Volume | Area_WP | LCB | LCF | Ixx | Iyy | WL_Length |
|---|---|---|---|---|---|---|---|---|---|
| Constrained_Randomized_Set_2 | 100 | 95 (95 %) | 97 | 100 | 100 | 100 | 98 | 100 | 100 |
| Constrained_Randomized_Set_3 | 100 | 98 (98 %) | 100 | 99 | 100 | 100 | 100 | 99 | 99 |
| Constrained_Randomized_Set_1 | 100 | 82 (82 %) | 90 | 97 | 99 | 97 | 98 | 96 | 95 |

| Set | Row | Outside the limit | Where |
|---|---|---|---|
| Constrained_Randomized_Set_2 | 3251 | Volume at T/Dd 0.12: OVD 0.001501, dataset 0.001468 (2.25 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_2 | 2740 | Ixx at T/Dd 0.12: OVD 0.000000, dataset 0.000000 (1.11 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_2 | 4751 | Ixx at T/Dd 0.12: OVD 0.000000, dataset 0.000000 (1.12 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_2 | 7596 | Volume at T/Dd 0.12: OVD 0.002764, dataset 0.002705 (2.18 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_2 | 9672 | Volume at T/Dd 0.25: OVD 0.001611, dataset 0.001635 (1.42 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_3 | 5310 | WL_Length at T/Dd 1.00: OVD 0.532639, dataset 0.523954 (1.66 x the limit); | deck level: the waterplane at the deck |
| Constrained_Randomized_Set_3 | 5710 | Area_WP at T/Dd 1.00: OVD 0.089457, dataset 0.090558 (1.22 x the limit); Iyy at T/Dd 1.00: OVD 0.001015, dataset 0.001051 (1.71 x the limit); | deck level: the waterplane at the deck |
| Constrained_Randomized_Set_1 | 1900 | Volume at T/Dd 0.25: OVD 0.004272, dataset 0.004320 (1.11 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_1 | 9764 | Volume at T/Dd 0.12: OVD 0.001998, dataset 0.002032 (1.67 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_1 | 6640 | Volume at T/Dd 0.12: OVD 0.000967, dataset 0.000955 (1.32 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_1 | 7207 | Area_WP at T/Dd 0.38: OVD 0.180512, dataset 0.175834 (2.66 x the limit); LCF at T/Dd 0.38: OVD 0.449821, dataset 0.460095 (2.05 x the limit); Iyy at T/Dd 0.38: OVD 0.009450, dataset 0.008737 (4.08 x the limit); | between |
| Constrained_Randomized_Set_1 | 2427 | WL_Length at T/Dd 0.12: OVD 0.685322, dataset 0.677088 (1.22 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_1 | 1857 | Iyy at T/Dd 1.00: OVD 0.002258, dataset 0.002317 (1.26 x the limit); | deck level: the waterplane at the deck |
| Constrained_Randomized_Set_1 | 5660 | Volume at T/Dd 0.25: OVD 0.006126, dataset 0.006048 (1.29 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_1 | 6102 | WL_Length at T/Dd 0.62: OVD 0.498689, dataset 0.492694 (1.22 x the limit); | between |
| Constrained_Randomized_Set_1 | 2007 | Volume at T/Dd 0.12: OVD 0.000329, dataset 0.000337 (2.30 x the limit); LCB at T/Dd 0.12: OVD 0.375044, dataset 0.381759 (1.34 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_1 | 5584 | Area_WP at T/Dd 1.00: OVD 0.163379, dataset 0.174939 (6.61 x the limit); LCF at T/Dd 1.00: OVD 0.461931, dataset 0.468370 (1.29 x the limit); Ixx at T/Dd 1.00: OVD 0.000526, dataset 0.000636 (8.64 x the limit); Iyy at T/Dd 1.00: OVD 0.009883, dataset 0.010312 (2.08 x the limit); | deck level: the waterplane at the deck |
| Constrained_Randomized_Set_1 | 5319 | Area_WP at T/Dd 1.00: OVD 0.196900, dataset 0.217625 (9.52 x the limit); LCF at T/Dd 1.00: OVD 0.492833, dataset 0.517583 (4.95 x the limit); Ixx at T/Dd 1.00: OVD 0.000683, dataset 0.000931 (13.35 x the limit); Iyy at T/Dd 1.00: OVD 0.017102, dataset 0.018647 (4.14 x the limit); | deck level: the waterplane at the deck |
| Constrained_Randomized_Set_1 | 7277 | Volume at T/Dd 0.12: OVD 0.001596, dataset 0.001577 (1.15 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_1 | 363 | WL_Length at T/Dd 0.12: OVD 1.040585, dataset 1.029588 (1.07 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_1 | 8107 | Volume at T/Dd 0.12: OVD 0.001774, dataset 0.001748 (1.45 x the limit); WL_Length at T/Dd 1.00: OVD 0.549669, dataset 0.543782 (1.08 x the limit); | between |
| Constrained_Randomized_Set_1 | 1533 | Volume at T/Dd 0.25: OVD 0.005909, dataset 0.005824 (1.48 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_1 | 853 | WL_Length at T/Dd 0.12: OVD 0.805082, dataset 0.794697 (1.31 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_1 | 9498 | Volume at T/Dd 0.12: OVD 0.001128, dataset 0.001142 (1.22 x the limit); | low draught: the immersed volume is small |
| Constrained_Randomized_Set_1 | 6203 | Volume at T/Dd 0.12: OVD 0.001419, dataset 0.001397 (1.57 x the limit); | low draught: the immersed volume is small |

## The fillet row

The quartic fillet between a bulb and the hull has one curvature condition. OVD writes it from the definition (no curvature at the bulb end). The dataset's code writes `6 x^2` where the second derivative has `6 x`. Of the 100 bulb hulls, 4 differ in volume by more than 0.1 % between the two rows (largest 0.279 %); each is listed, with whether it passes under each row.

| Set | Row | Volume difference | Definition | Dataset row |
|---|---|---|---|---|
| Constrained_Randomized_Set_1 | 5117 | 0.174 % | pass | pass |
| Constrained_Randomized_Set_1 | 3767 | 0.101 % | pass | pass |
| Constrained_Randomized_Set_1 | 1849 | 0.135 % | pass | pass |
| Constrained_Randomized_Set_1 | 920 | 0.279 % | pass | pass |

## At a Design's cut

For information: a Design's ShipD hull is cut more coarsely (41 even stations and 20 even height levels, no waterline-end stations), so its hydrostatics and GZ curves stay fast. At that cut OVD's own integration gives:

| Set | Hulls | Pass | Volume | Area_WP | LCB | LCF | Ixx | Iyy | WL_Length |
|---|---|---|---|---|---|---|---|---|---|
| Constrained_Randomized_Set_2 | 100 | 0 (0 %) | 33 | 44 | 83 | 75 | 38 | 24 | 0 |
| Constrained_Randomized_Set_3 | 100 | 0 (0 %) | 16 | 71 | 97 | 82 | 69 | 39 | 0 |
| Constrained_Randomized_Set_1 | 100 | 0 (0 %) | 34 | 56 | 85 | 76 | 48 | 37 | 0 |
