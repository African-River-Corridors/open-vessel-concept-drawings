//! Seam: `ovd_core::shipd`, the ShipD-style parametric hull (issues #452 and #453, spec 0008
//! "ShipD-style hull").
//!
//! Label: sourced - the Ship-D dataset (Bagazinski and Ahmed 2024, Harvard Dataverse,
//! doi:10.7910/DVN/MMGAUS, V1, CC0 1.0): the input vectors and the measures of the committed
//! sample in `calibration/shipd/` (see its `SOURCE.md`), and the 49 constraints of
//! `HullParameterization_ConstraintsOverview.pdf`. The constraint tests are properties: each
//! vector breaks exactly the constraint it is named for. The sections of every sampled hull are a
//! property of the generator, not a sourced number.

use ovd_core::shipd::{
    BENCHMARK, CONSTRAINTS, DESIGN, DRAUGHT_FRACTIONS, FilletRow, Measures, NAMES, Reading,
    Resolution, Shape, ShipdError, ShipdParameters, check, dataset_measures, measures, sections,
};

/// The set of the sample that has bulbs.
const BULB_SET: &str = "Constrained_Randomized_Set_1";

const SAMPLE: &str = include_str!("../../../calibration/shipd/sample.csv");
const REFERENCE: &str = include_str!("../../../calibration/shipd/reference.csv");
const ARTEFACTS: &str = include_str!("../../../calibration/shipd/artefacts.csv");

/// Row `n` (from 0) of the committed sample: its set, its dataset row and its 45 parameters.
fn sample(n: usize) -> (String, String, [f64; 45]) {
    let line = SAMPLE.lines().nth(n + 1).unwrap();
    let cells: Vec<&str> = line.split(',').collect();
    let v: Vec<f64> = cells[2..].iter().map(|c| c.parse().unwrap()).collect();
    (
        cells[0].to_string(),
        cells[1].to_string(),
        v.try_into().unwrap(),
    )
}

fn dims(v: &[f64; 45]) -> (f64, f64, f64) {
    (v[0], v[3] * v[0], v[4] * v[0])
}

fn failures(v: [f64; 45]) -> Vec<usize> {
    let (l, b, d) = dims(&v);
    check(&ShipdParameters(v), l, b, d)
        .iter()
        .map(|f| f.index)
        .collect()
}

#[test]
fn the_header_names_are_the_datasets() {
    let header: Vec<&str> = SAMPLE.lines().next().unwrap().split(',').skip(2).collect();
    assert_eq!(header, NAMES);
    assert_eq!(CONSTRAINTS.len(), 49);
}

/// A constraint, the sample hull (0 or 1) and the parameters changed so that it alone fails.
type Break = (usize, usize, &'static [(usize, f64)]);

/// For each constraint that can fail alone: its vector.
const BREAKS: [Break; 33] = [
    (0, 0, &[(1, 0.54)]),
    (1, 0, &[(6, 1.01)]),
    (2, 0, &[(3, -4.0), (7, 0.6)]),
    (3, 0, &[(8, 69.0), (9, -4.0)]),
    (5, 0, &[(8, 99.0), (9, 0.0)]),
    (6, 0, &[(7, 0.2), (8, 69.0)]),
    (7, 0, &[(4, 0.47)]),
    (8, 0, &[(10, 0.0)]),
    (9, 0, &[(14, 1.01)]),
    (10, 1, &[(18, -2.6), (19, 179.0)]),
    (11, 0, &[(19, -4.0)]),
    (12, 0, &[(17, -1.73)]),
    (13, 0, &[(17, -4.0)]),
    (14, 0, &[(17, -2.84)]),
    (15, 0, &[(17, 0.8), (18, -1.2)]),
    (17, 0, &[(11, -2.42)]),
    (18, 0, &[(13, -1.04)]),
    (19, 0, &[(13, 1.01)]),
    (20, 0, &[(11, 2.98)]),
    (21, 0, &[(11, -2.3), (15, 1.7)]),
    (22, 0, &[(11, -2.34)]),
    (23, 0, &[(11, -51.0), (12, 89.0)]),
    (24, 0, &[(24, -4.0)]),
    (25, 0, &[(23, 1.4), (27, 129.0)]),
    (26, 1, &[(22, 1.8), (26, 0.8)]),
    (27, 0, &[(22, -4.0)]),
    (28, 0, &[(2, 0.07)]),
    (29, 0, &[(3, -1.06)]),
    (30, 0, &[(27, 145.0)]),
    (31, 0, &[(3, 0.2), (29, -4.0)]),
    (33, 1, &[(4, 1.3), (23, -0.1)]),
    (34, 0, &[(4, 0.09)]),
    (35, 0, &[(28, 0.0)]),
];

#[test]
fn each_constraint_fails_alone_for_its_vector() {
    for (index, row, changes) in BREAKS {
        let (_, _, mut v) = sample(row);
        assert!(failures(v).is_empty(), "sample hull {row} passes");
        for &(j, x) in changes {
            v[j] = x;
        }
        assert_eq!(
            failures(v),
            vec![index],
            "{index} ({}): {changes:?}",
            CONSTRAINTS[index]
        );
    }
}

#[test]
fn a_negative_chine_breadth_fails_with_the_chine_constraints_it_forces() {
    // Constraints 4 and 32 cannot fail alone: a negative chine half-breadth makes the chine radius
    // (a share of it) negative too, and puts the chine fillet outboard of the chine.
    let (_, _, mut v) = sample(0);
    v[7] = -0.01;
    assert_eq!(failures(v), vec![2, 3, 4, 6, 7]);
    let (_, _, mut v) = sample(0);
    v[28] = -0.01;
    assert_eq!(failures(v), vec![30, 31, 32, 34, 35]);
}

#[test]
fn the_bow_keel_intersection_is_never_aft_of_the_stem() {
    // Constraint 16 (BK_x >= 0) cannot fail: the bow rake's constant puts its least x, which
    // includes the rake at BK_z, at 0. Swept over the bow rake terms of the sample hulls.
    for n in [0, 1, 100, 101] {
        let (_, _, base) = sample(n);
        for a in -8..=8 {
            for b in -8..=8 {
                let mut v = base;
                v[11] = f64::from(a) * 0.5;
                v[12] = f64::from(b) * 0.5;
                let (l, br, d) = dims(&v);
                let g = Shape::new(&ShipdParameters(v), l, br, d).constraints();
                assert!(g[16] <= 0.0, "{n}: Abow {a}, Bbow {b}: {}", g[16]);
            }
        }
    }
}

#[test]
fn without_bulbs_the_bulb_constraints_hold_and_a_constraint_failure_says_which() {
    let (_, _, v) = sample(0);
    let (l, b, d) = dims(&v);
    let g = Shape::new(&ShipdParameters(v), l, b, d).constraints();
    assert!(g[36..].iter().all(|&x| x == -1.0));
    let mut bad = v;
    bad[1] = 0.54;
    match sections(&ShipdParameters(bad), l, b, d, DESIGN) {
        Err(ShipdError::Constraints(c)) => {
            assert_eq!(c.len(), 1);
            assert_eq!(c[0].index, 0);
            assert!(format!("{}", ShipdError::Constraints(c)).contains("constraint 0"));
        }
        other => panic!("{other:?}"),
    }
}

/// The sampled hull of the bulb set with this dataset row (from 0).
fn bulb_hull(row: &str) -> [f64; 45] {
    SAMPLE
        .lines()
        .skip(1)
        .map(|l| l.split(',').collect::<Vec<_>>())
        .find(|c| c[0] == BULB_SET && c[1] == row)
        .map(|c| {
            c[2..]
                .iter()
                .map(|x| x.parse().unwrap())
                .collect::<Vec<f64>>()
        })
        .unwrap()
        .try_into()
        .unwrap()
}

/// A bulb hull of the sample that passes all 49 constraints, with both bulbs, a keel radius and a
/// deadrise: the base of the vectors below.
const BULB_BASE: &str = "763";

/// For each bulb constraint (36 to 48): the parameters changed so that it alone fails. They are
/// properties of the constraint, not values of a hull: some are outside the dataset's ranges.
const BULB_BREAKS: [(usize, &[(usize, f64)]); 13] = [
    // Lower vertical radius of the bow bulb above the keel radius.
    (36, &[(6, 0.45), (34, 1.95)]),
    // Length radius of the bow bulb above the keel radius.
    (37, &[(10, 0.3), (34, 0.45)]),
    // Bow bulb wider than the hull at its widest.
    (38, &[(35, 1.05)]),
    // Bow bulb aft of the midship breadth at the base.
    (
        39,
        &[
            (15, -0.382_917_448_672_108),
            (16, 0.188_279_147_586_32),
            (36, -0.179_374_403_752_875),
            (33, 0.987_446_092_802_853),
            (6, 0.382_095_008_535_314),
        ],
    ),
    // Bow bulb aft of the midship breadth at its vertex.
    (40, &[(1, 0.2), (6, 0.6)]),
    // Bow bulb aft of the midship breadth at the design draught.
    (41, &[(1, 0.15)]),
    // Lower vertical radius of the stern bulb above the keel radius.
    (
        42,
        &[
            (10, 0.065_632_494_589_89),
            (41, 0.754_085_010_252_78),
            (42, 0.054_612_292_726_761),
        ],
    ),
    // Length radius of the stern bulb above the keel radius.
    (43, &[(10, 0.05)]),
    // Stern bulb wider than the hull at its widest.
    (44, &[(42, 1.05)]),
    // Stern bulb higher than the stern-keel intersection.
    (45, &[(23, 0.0)]),
    // Stern bulb forward of the midship breadth at the base.
    (46, &[(38, -0.7), (40, -1.0)]),
    // Stern bulb forward of the midship breadth at its height.
    (47, &[(6, 0.65)]),
    // Stern bulb forward of the midship breadth at its vertex.
    (
        48,
        &[
            (24, 0.271_969_637_418_438),
            (25, -0.757_362_170_835_696),
            (26, 0.233_209_404_998_068),
            (38, 0.287_677_963_188_439),
            (43, -0.126_428_023_088_338),
            (39, 0.081_331_340_369_569_6),
        ],
    ),
];

#[test]
fn each_bulb_constraint_fails_alone_for_its_vector() {
    let base = bulb_hull(BULB_BASE);
    assert!(failures(base).is_empty(), "the base passes");
    for (index, changes) in BULB_BREAKS {
        let mut v = base;
        for &(j, x) in changes {
            v[j] = x;
        }
        assert_eq!(
            failures(v),
            vec![index],
            "{index} ({}): {changes:?}",
            CONSTRAINTS[index]
        );
    }
}

#[test]
fn a_bulb_needs_a_zero_deadrise_or_a_positive_keel_radius() {
    // With a deadrise and a keel radius that is not positive, every bulb row that compares with
    // the keel fails (the dataset's code does the same by branch); with no deadrise, none does.
    let base = bulb_hull(BULB_BASE);
    let mut v = base;
    v[10] = -0.3;
    assert_eq!(failures(v), vec![36, 37, 38, 42, 43, 44, 45]);
    let mut v = base;
    v[8] = 0.0;
    assert!(failures(v).is_empty());
    v[10] = -0.3;
    assert!(
        !failures(v).iter().any(|i| (36..=38).contains(i)),
        "{:?}",
        failures(v)
    );
}

#[test]
fn a_bulb_hull_sections_are_closed_and_simple_at_every_station() {
    // Every hull of the bulb set in the sample, at a Design's cut: `SectionsTable::new` refuses a
    // section that crosses itself, has a negative half-breadth or does not start and end on the
    // centre line.
    let mut n = 0;
    for line in SAMPLE.lines().skip(1) {
        let c: Vec<&str> = line.split(',').collect();
        if c[0] != BULB_SET {
            continue;
        }
        let v: Vec<f64> = c[2..].iter().map(|x| x.parse().unwrap()).collect();
        let v: [f64; 45] = v.try_into().unwrap();
        let (l, b, d) = dims(&v);
        let t = sections(&ShipdParameters(v), l, b, d, DESIGN)
            .unwrap_or_else(|e| panic!("{} {}: {e}", c[0], c[1]));
        let last = t.sections[0].points.len() - 1;
        for s in &t.sections {
            assert_eq!(s.points[last], (0.0, d), "{} {}", c[0], c[1]);
            assert!(s.points.iter().all(|p| p.0 >= 0.0 && p.0.is_finite()));
        }
        assert!(t.sections.windows(2).all(|w| w[0].x < w[1].x));
        n += 1;
    }
    assert_eq!(n, 100);
}

#[test]
fn a_bulb_leads_the_stem_and_trails_the_stern_by_its_length() {
    // At the height of its widest part the bow bulb's leading edge is `Lbb LOA` forward of the stem
    // at the design draught, and the stern bulb's trailing edge is `Lsb LOA` aft of where it starts.
    // The tolerance is rounding only.
    let v = bulb_hull(BULB_BASE);
    let (l, b, d) = dims(&v);
    let shape = Shape::new(&ShipdParameters(v), l, b, d);
    let wl = v[6] * d;
    let fp = shape.bow_profile(wl);
    let bow = shape.waterline(v[34] * wl, FilletRow::Definition);
    assert!(
        (bow.span().0 - (fp - l * v[33])).abs() < 1e-9,
        "{:?}",
        bow.span()
    );
    let top = wl * v[40];
    let start = v[38] * v[2] * l + (l - v[1] * l - v[2] * l) + v[1] * l;
    let stern = shape.waterline(v[41] * top, FilletRow::Definition);
    assert!(
        (stern.span().1 - (start + l * v[39])).abs() < 1e-9,
        "{:?}",
        stern.span()
    );
    // The Sections Table has a station at each of those two x.
    let t = sections(&ShipdParameters(v), l, b, d, DESIGN).unwrap();
    for x in [fp - l * v[33], start + l * v[39]] {
        let at = l - x;
        assert!(t.sections.iter().any(|s| (s.x - at).abs() < 1e-9), "{at}");
    }
}

/// The dataset's measures of a sampled hull, per draught, in the `Measures` field order.
fn reference(set: &str, row: &str) -> Vec<[f64; 7]> {
    let of = |q: &str| -> Vec<f64> {
        let line = REFERENCE
            .lines()
            .find(|l| l.starts_with(&format!("{set},{row},{q},")))
            .unwrap();
        line.split(',')
            .skip(3)
            .map(|c| c.parse().unwrap())
            .collect()
    };
    let cols: Vec<Vec<f64>> = ["Volume", "Area_WP", "LCB", "LCF", "Ixx", "Iyy", "WL_Length"]
        .iter()
        .map(|q| of(q))
        .collect();
    (0..10)
        .map(|k| [0, 1, 2, 3, 4, 5, 6].map(|q| cols[q][k]))
        .collect()
}

#[test]
fn ten_sampled_hulls_agree_with_the_dataset_within_the_benchmark_limits() {
    // The first five hulls of each set in the committed sample, at the benchmark's cut; the full sample runs in `cargo run -p xtask -- shipd-benchmark`.
    let limits = [
        (0.01, true),
        (0.01, true),
        (0.005, false),
        (0.005, false),
        (0.02, true),
        (0.02, true),
        (0.01, true),
    ];
    for n in [0, 1, 2, 3, 4, 100, 101, 102, 103, 104] {
        let (set, row, v) = sample(n);
        let (l, b, d) = dims(&v);
        let table = sections(&ShipdParameters(v), l, b, d, BENCHMARK).unwrap();
        let draughts: Vec<f64> = DRAUGHT_FRACTIONS
            .iter()
            .map(|f| f.min(1.0 - 1e-9) * d)
            .collect();
        let ours = measures(&table, l, &draughts);
        for (k, (o, r)) in ours.iter().zip(reference(&set, &row)).enumerate() {
            let o = [
                o.volume,
                o.waterplane_area,
                o.lcb,
                o.lcf,
                o.ixx,
                o.iyy,
                o.waterline_length,
            ];
            for q in 0..7 {
                let (limit, relative) = limits[q];
                let diff = if relative {
                    (o[q] - r[q]).abs() / r[q].abs()
                } else {
                    (o[q] - r[q]).abs()
                };
                assert!(
                    diff <= limit,
                    "{set} {row}, quantity {q} at T/Dd {}: OVD {} vs {}",
                    DRAUGHT_FRACTIONS[k],
                    o[q],
                    r[q]
                );
            }
        }
    }
}

#[test]
fn the_table_is_closed_and_its_stations_run_aft_to_forward() {
    let (_, _, v) = sample(0);
    let (l, b, d) = dims(&v);
    let t = sections(&ShipdParameters(v), l, b, d, DESIGN).unwrap();
    let n = t.sections[0].points.len();
    assert!(t.sections.iter().all(|s| s.points.len() == n));
    assert!(t.sections.windows(2).all(|w| w[0].x <= w[1].x));
    // The deck at the centre line closes every section at the depth.
    assert!(t.sections.iter().all(|s| s.points[n - 1] == (0.0, d)));
    // Volume grows with the draught.
    let m = measures(&t, l, &[0.2 * d, 0.5 * d, 0.9 * d]);
    assert!(m[0].volume < m[1].volume && m[1].volume < m[2].volume);
}

/// The worst difference of `ours` from the dataset's measures of `set` `row`, as a share of the
/// benchmark's limit (above 1 is outside).
fn worst_share(ours: &[Measures], set: &str, row: &str) -> f64 {
    let limits = [
        (0.01, true),
        (0.01, true),
        (0.005, false),
        (0.005, false),
        (0.02, true),
        (0.02, true),
        (0.01, true),
    ];
    let mut worst = 0.0_f64;
    for (o, r) in ours.iter().zip(reference(set, row)) {
        let o = [
            o.volume,
            o.waterplane_area,
            o.lcb,
            o.lcf,
            o.ixx,
            o.iyy,
            o.waterline_length,
        ];
        for q in 0..7 {
            let (limit, relative) = limits[q];
            let diff = if relative {
                (o[q] - r[q]).abs() / r[q].abs()
            } else {
                (o[q] - r[q]).abs()
            };
            worst = worst.max(diff / limit);
        }
    }
    worst
}

/// The parameters of the sampled hull `set` `row`.
fn sampled(set: &str, row: &str) -> [f64; 45] {
    SAMPLE
        .lines()
        .skip(1)
        .map(|l| l.split(',').collect::<Vec<_>>())
        .find(|c| c[0] == set && c[1] == row)
        .map(|c| {
            c[2..]
                .iter()
                .map(|x| x.parse().unwrap())
                .collect::<Vec<f64>>()
                .try_into()
                .unwrap()
        })
        .unwrap()
}

#[test]
fn fifteen_sampled_hulls_pass_the_shape_gate() {
    // The gate of the benchmark (#648): the shape measured as the dataset measures, on the first five
    // hulls of each set without bulbs and five bulb hulls; the full sample runs in the xtask.
    for n in [
        0, 1, 2, 3, 4, 100, 101, 102, 103, 104, 200, 201, 202, 203, 204,
    ] {
        let (set, row, v) = sample(n);
        let (l, b, d) = dims(&v);
        let w = worst_share(
            &dataset_measures(&ShipdParameters(v), l, b, d, BENCHMARK),
            &set,
            &row,
        );
        assert!(w <= 1.0, "{set} {row}: {w:.3} x the limit");
    }
}

#[test]
fn each_listed_artefact_misses_by_the_definitions_and_the_dataset_code_reading_reproduces_it() {
    // `calibration/shipd/artefacts.csv` (#648): a hull is excused from the gate only with this
    // evidence. By the definitions it is outside the limits; read as the dataset's code reads the
    // one place named by its cause, it is within them.
    let mut n = 0;
    for line in ARTEFACTS.lines().skip(1).filter(|l| !l.trim().is_empty()) {
        let c: Vec<&str> = line.split(',').collect();
        let (set, row, cause) = (c[0], c[1], c[2]);
        let v = sampled(set, row);
        let (l, b, d) = dims(&v);
        let p = ShipdParameters(v);
        let read = match cause {
            "stern_ellipse" => Resolution {
                stern_ellipse: Reading::DatasetCode,
                ..BENCHMARK
            },
            "bulb_crossing" => Resolution {
                bulb_crossing: Reading::DatasetCode,
                ..BENCHMARK
            },
            _ => panic!("unknown cause {cause}"),
        };
        let by_definitions = worst_share(&dataset_measures(&p, l, b, d, BENCHMARK), set, row);
        let by_code = worst_share(&dataset_measures(&p, l, b, d, read), set, row);
        assert!(
            by_definitions > 1.0,
            "{set} {row}: within the limits by the definitions"
        );
        assert!(
            by_code <= 1.0,
            "{set} {row}: {cause} does not reproduce it ({by_code:.3})"
        );
        n += 1;
    }
    assert_eq!(n, 6);
}
