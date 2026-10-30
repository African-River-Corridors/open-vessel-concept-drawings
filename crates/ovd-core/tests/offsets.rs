//! Seam: `ovd_core::offsets`, the typed offsets table (issue #451, spec 0008 "Offsets table").
//!
//! Label: property. Each test states a rule: the round trip gives the same table, each bad table is
//! refused with its reason, and a resampled table keeps its own points and its volume.

use ovd_core::hull::hull_form::{HullFormInputs, HullFormKind, HullShape, generate};
use ovd_core::offsets::{OffsetRow, OffsetsError, dimensions, from_rows, import, to_rows};
use ovd_core::sections::{Waterplane, immersed};

const L: f64 = 60.0;
const B: f64 = 10.0;
const D: f64 = 4.0;

fn raked() -> ovd_core::sections::SectionsTable {
    generate(&HullFormInputs {
        length_overall_m: L,
        beam_m: B,
        moulded_depth_m: D,
        form: HullFormKind::RakedBox {
            bow_rake_length_m: 6.0,
            bow_rake_rise_m: 2.0,
            stern_rake_length_m: 4.0,
            stern_rake_rise_m: 1.5,
        },
        shape: HullShape {
            bilge_radius_m: 0.8,
            camber_m: 0.2,
            ..HullShape::default()
        },
    })
    .unwrap()
}

fn row(x: f64, y: f64, z: f64) -> OffsetRow {
    OffsetRow {
        station_x_m: x,
        y_m: y,
        z_m: z,
    }
}

#[test]
fn a_raked_box_table_survives_the_round_trip() {
    let t = raked();
    let back = from_rows(&to_rows(&t)).unwrap();
    assert_eq!(back, t);
    let d = dimensions(&back);
    assert_eq!((d.length_m, d.beam_m, d.depth_m), (L, B, D));
}

fn box_station(x: f64) -> Vec<OffsetRow> {
    vec![
        row(x, 0.0, 0.0),
        row(x, B / 2.0, 0.0),
        row(x, B / 2.0, D),
        row(x, 0.0, D),
    ]
}

#[test]
fn each_bad_table_gives_its_reason() {
    let mut one = box_station(0.0);
    assert_eq!(
        from_rows(&one),
        Err(OffsetsError::TooFewStations { count: 1 })
    );
    let mut back = box_station(10.0);
    back.extend(box_station(0.0));
    assert_eq!(
        from_rows(&back),
        Err(OffsetsError::XDescending { station: 1 })
    );
    one.extend(vec![row(10.0, 0.0, 0.0)]);
    assert_eq!(
        from_rows(&one),
        Err(OffsetsError::TooFewPoints { station: 1 })
    );
    let mut off = box_station(0.0);
    off.extend(vec![
        row(10.0, 0.0, 0.0),
        row(10.0, B / 2.0, 0.0),
        row(10.0, B / 2.0, D),
    ]);
    assert_eq!(
        from_rows(&off),
        Err(OffsetsError::OffCentreLine { station: 1 })
    );
    let mut port = box_station(0.0);
    port.extend(box_station(10.0));
    port[5].y_m = -1.0;
    assert_eq!(
        from_rows(&port),
        Err(OffsetsError::NegativeY { station: 1 })
    );
    let mut crossed = box_station(0.0);
    crossed.extend(vec![
        row(10.0, 0.0, 0.0),
        row(10.0, B / 2.0, D),
        row(10.0, B / 2.0, 0.0),
        row(10.0, 0.0, D),
    ]);
    assert_eq!(
        from_rows(&crossed),
        Err(OffsetsError::SelfCrossing { station: 1 })
    );
    let mut nan = box_station(0.0);
    nan.extend(box_station(10.0));
    nan[2].z_m = f64::NAN;
    assert_eq!(from_rows(&nan), Err(OffsetsError::NotFinite { row: 2 }));
}

#[test]
fn unequal_stations_are_resampled_keep_their_points_and_their_volume() {
    // The aft station is the plain box; the forward one has the same shape with more points on the
    // bottom and the side.
    let mut rows = box_station(0.0);
    rows.extend(vec![
        row(L, 0.0, 0.0),
        row(L, 2.0, 0.0),
        row(L, B / 2.0, 0.0),
        row(L, B / 2.0, 1.0),
        row(L, B / 2.0, D),
        row(L, 0.0, D),
    ]);
    let i = import(&rows).unwrap();
    assert_eq!(i.resampled, vec![0]);
    assert_eq!(i.points, 6);
    let aft = &i.table.sections[0].points;
    for p in [(0.0, 0.0), (B / 2.0, 0.0), (B / 2.0, D), (0.0, D)] {
        assert!(aft.contains(&p), "the corner {p:?} is kept: {aft:?}");
    }
    for draught in [0.5, 2.0, 3.5] {
        let v = immersed(&i.table, &Waterplane::upright(draught)).volume_m3;
        let typed = L * B * draught;
        assert!(
            (v - typed).abs() <= 1e-3 * typed,
            "{draught} m: {v} vs {typed}"
        );
    }
}
