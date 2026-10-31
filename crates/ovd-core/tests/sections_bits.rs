//! Seam: `ovd_core::sections`, the bits of the polygon integrator (issue #567).
//!
//! Label: snapshot - each pinned fingerprint is what the integrator printed at `67db6f9`, before the
//! speed work of issue #567. A speed-up of the cut or the integration must not move one bit of any
//! result, so every output is folded in by its bit pattern, not compared within a tolerance.
//!
//! The tables cover a ShipD hull at a Design's cut (many stations, many points), a box, a raked hull
//! with a step, and a slotted (ATB Notch) table. The waterplanes cover heels from 0 to 60 degrees,
//! trims by the stern and the bow, and heights from the keel to above the deck, so cuts with no
//! crossing, with several crossings and with a slot all occur.

use ovd_core::sections::{
    Gravity, Section, SectionsTable, Waterplane, draught_for_volume, equilibrium, gz_curve,
    immersed,
};
use ovd_core::shipd::{DESIGN, ShipdParameters, sections};

const SAMPLE: &str = include_str!("../../../calibration/shipd/sample.csv");

/// FNV-1a over the bit patterns of the numbers.
struct Fingerprint(u64);

impl Fingerprint {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    fn add(&mut self, v: f64) {
        for byte in v.to_bits().to_le_bytes() {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
        }
    }
}

fn shipd_table() -> SectionsTable {
    let line = SAMPLE.lines().nth(1).unwrap();
    let v: Vec<f64> = line
        .split(',')
        .skip(2)
        .map(|c| c.parse().unwrap())
        .collect();
    let v: [f64; 45] = v.try_into().unwrap();
    sections(&ShipdParameters(v), v[0], v[3] * v[0], v[4] * v[0], DESIGN).unwrap()
}

fn sec(x: f64, points: Vec<(f64, f64)>) -> Section {
    Section { x, points }
}

fn box_half(b: f64, d: f64) -> Vec<(f64, f64)> {
    vec![(0.0, 0.0), (b / 2.0, 0.0), (b / 2.0, d), (0.0, d)]
}

fn box_table() -> SectionsTable {
    SectionsTable::new(
        vec![
            sec(0.0, box_half(11.4, 4.0)),
            sec(60.0, box_half(11.4, 4.0)),
        ],
        vec![4.0, 4.0],
    )
    .unwrap()
}

/// A raked bow, a parallel body and a transom step.
fn raked_table() -> SectionsTable {
    let (b, d) = (10.0, 3.5);
    let chine = vec![
        (0.0, 0.0),
        (b / 2.0 - 0.6, 0.0),
        (b / 2.0, 0.6),
        (b / 2.0, d),
        (0.0, d),
    ];
    let chine_bow = vec![
        (0.0, 2.0),
        (b / 2.0 - 0.6, 2.0),
        (b / 2.0, 2.1),
        (b / 2.0, d),
        (0.0, d),
    ];
    let stern = vec![
        (0.0, 0.8),
        (b / 2.0 - 0.6, 0.8),
        (b / 2.0, 1.0),
        (b / 2.0, d),
        (0.0, d),
    ];
    SectionsTable::new(
        vec![
            sec(0.0, stern.clone()),
            sec(4.0, chine.clone()),
            sec(44.0, chine),
            sec(50.0, chine_bow),
        ],
        vec![d; 4],
    )
    .unwrap()
}

/// An ATB Notch: slotted at the aft end, a step, then whole.
fn slotted_table() -> SectionsTable {
    let (b, d) = (11.4, 4.0);
    let slot = vec![(1.0, 0.0), (b / 2.0, 0.0), (b / 2.0, d), (1.0, d)];
    SectionsTable::new(
        vec![
            sec(0.0, slot.clone()),
            sec(10.0, slot),
            sec(10.0, box_half(b, d)),
            sec(60.0, box_half(b, d)),
        ],
        vec![d; 4],
    )
    .unwrap()
}

/// Every output of `immersed` over a grid of waterplanes.
fn immersed_fingerprint(t: &SectionsTable, depth: f64) -> u64 {
    let mut fp = Fingerprint::new();
    for heel in [0.0, 3.0, 11.5, 25.0, 40.0, 60.0] {
        for trim in [-4.0, -0.5, 0.0, 1.25, 6.0] {
            for k in 0..=9 {
                let c = -0.5 + f64::from(k) * (depth + 1.5) / 9.0;
                let im = immersed(
                    t,
                    &Waterplane {
                        heel_deg: heel,
                        trim_deg: trim,
                        c,
                    },
                );
                for v in [
                    im.volume_m3,
                    im.lcb_m,
                    im.tcb_m,
                    im.kb_m,
                    im.waterplane_area_m2,
                    im.lcf_m,
                    im.tcf_m,
                    im.vcf_m,
                    im.i_t_m4,
                    im.i_l_m4,
                    im.wetted_surface_m2,
                    im.l_wl_m,
                    im.b_wl_m,
                    im.midship_area_m2,
                ] {
                    fp.add(v);
                }
            }
        }
    }
    fp.0
}

/// The free-trim solves: the level draught, equilibria at several heels and a GZ curve.
fn solve_fingerprint(t: &SectionsTable, mass: f64, g: Gravity) -> u64 {
    let mut fp = Fingerprint::new();
    fp.add(draught_for_volume(t, mass).unwrap());
    for heel in [0.0, 9.0, 33.0] {
        let eq = equilibrium(t, mass, g, heel, 1.0).unwrap();
        for v in [
            eq.waterplane.c,
            eq.waterplane.trim_deg,
            eq.immersed.volume_m3,
            eq.immersed.lcb_m,
            eq.immersed.tcb_m,
            eq.immersed.kb_m,
        ] {
            fp.add(v);
        }
    }
    for (deg, gz) in gz_curve(t, mass, g, 120.0, 1.0, (0..=60).step_by(5)).unwrap() {
        fp.add(f64::from(deg));
        fp.add(gz);
    }
    fp.0
}

#[test]
fn the_cut_of_every_table_keeps_its_bits() {
    let shipd = shipd_table();
    let depth = shipd
        .sections
        .iter()
        .flat_map(|s| s.points.iter().map(|p| p.1))
        .fold(0.0, f64::max);
    let got = [
        immersed_fingerprint(&shipd, depth),
        immersed_fingerprint(&box_table(), 4.0),
        immersed_fingerprint(&raked_table(), 3.5),
        immersed_fingerprint(&slotted_table(), 4.0),
    ];
    println!("immersed fingerprints: {got:#018x?}");
    assert_eq!(got, PINNED_IMMERSED);
}

#[test]
fn the_free_trim_solves_keep_their_bits() {
    let shipd = shipd_table();
    let x_mid = (shipd.sections[0].x + shipd.sections.last().unwrap().x) / 2.0;
    let total = immersed(&shipd, &Waterplane::upright(f64::MAX / 4.0)).volume_m3;
    let got = [
        solve_fingerprint(
            &shipd,
            0.45 * total,
            Gravity {
                lcg_m: x_mid,
                tcg_m: 0.0,
                kg_m: 0.4 * shipd.sections[0].points.last().unwrap().1.max(1.0),
            },
        ),
        solve_fingerprint(
            &raked_table(),
            900.0,
            Gravity {
                lcg_m: 26.0,
                tcg_m: 0.0,
                kg_m: 2.1,
            },
        ),
        solve_fingerprint(
            &slotted_table(),
            1300.0,
            Gravity {
                lcg_m: 33.0,
                tcg_m: 0.0,
                kg_m: 2.4,
            },
        ),
    ];
    println!("solve fingerprints: {got:#018x?}");
    assert_eq!(got, PINNED_SOLVES);
}

const PINNED_IMMERSED: [u64; 4] = [
    0xe3fa_592b_0d2e_d179,
    0x0154_5546_c697_78d0,
    0x8701_d11f_22dd_2e15,
    0x1d41_6168_6a63_dd25,
];
const PINNED_SOLVES: [u64; 3] = [
    0xc1fb_bcfa_cba7_7828,
    0x52d9_663d_af9b_759c,
    0x45e2_f9a6_8efe_f925,
];
