//! Seam: `ovcd_core::shipd`, the ShipD-style parametric hull without bulbs (issue #452, spec 0008
//! "ShipD-style hull").
//!
//! Label: sourced - the Ship-D dataset (Bagazinski and Ahmed 2024, Harvard Dataverse,
//! doi:10.7910/DVN/MMGAUS, V1, CC0 1.0): the input vectors and the measures of the committed
//! sample in `calibration/shipd/` (see its `SOURCE.md`), and the 49 constraints of
//! `HullParameterization_ConstraintsOverview.pdf`. The constraint tests are properties: each
//! vector breaks exactly the constraint it is named for.

use ovcd_core::shipd::{
    BENCHMARK, CONSTRAINTS, DESIGN, DRAUGHT_FRACTIONS, NAMES, Shape, ShipdError, ShipdParameters,
    check, measures, sections,
};

const SAMPLE: &str = include_str!("../../../calibration/shipd/sample.csv");
const REFERENCE: &str = include_str!("../../../calibration/shipd/reference.csv");

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
fn without_bulbs_the_bulb_constraints_hold_and_a_bulb_is_refused() {
    let (_, _, v) = sample(0);
    let (l, b, d) = dims(&v);
    let g = Shape::new(&ShipdParameters(v), l, b, d).constraints();
    assert!(g[36..].iter().all(|&x| x == -1.0));
    let mut bulb = v;
    bulb[31] = 1.0;
    assert_eq!(
        sections(&ShipdParameters(bulb), l, b, d, DESIGN),
        Err(ShipdError::Bulb)
    );
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
                    "{set} {row}, quantity {q} at T/Dd {}: OVCD {} vs {}",
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
