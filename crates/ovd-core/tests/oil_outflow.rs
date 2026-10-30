//! Seam: `ovd_core::liquid_bulk::oil_outflow`, the mean oil outflow parameter of MARPOL Annex I regulation 23
//! (issue #341).
//!
//! Label: sourced - Res. MEPC.122(52), the explanatory notes, part C 1 "Tank barge example": a barge L = 100 m,
//! BS = BB = 40 m, DS = 20 m, load line draught 9.0 m, deadweight 33,949 t, two cargo tanks CO1 (20 to 35 m
//! from the aft perpendicular) and CO2 (35 to 80 m), each 36 m wide and from 2.0 to 20.0 m above the base line,
//! an inert gas overpressure of 5 kPa. The notes give C = 37,721 m3, the nominal density 900 kg/m3, the
//! probabilities of each tank, the cargo level after the damage (7.406 m and 4.559 m), the outflows of each
//! tank, OMS = 4,172 m3, OMB(0) = 1,544 m3, OMB(2.5) = 1,974 m3, OMB = 1,673 m3 and OM = 0.071. The notes round
//! their probabilities to 3 decimals, so the figures here agree to the rounding of the notes (0.5 %).

use ovd_core::compartments::{Side, Space, SpaceKind, SpaceShape};
use ovd_core::liquid_bulk::oil_outflow::{OilOutflowInputs, oil_outflow, om_limit};

fn tank(name: &str, x: (f64, f64)) -> Space {
    Space {
        name: name.to_string(),
        kind: SpaceKind::CargoTank,
        side: Side::Centre,
        x_aft: x.0,
        x_fwd: x.1,
        shape: SpaceShape::Prism {
            section: vec![(-18.0, 2.0), (18.0, 2.0), (18.0, 20.0), (-18.0, 20.0)],
        },
    }
}

fn barge() -> OilOutflowInputs {
    OilOutflowInputs {
        length_m: 100.0,
        beam_m: 40.0,
        depth_m: 20.0,
        draught_m: 9.0,
        deadweight_t: 33_949.0,
        overpressure_kpa: 5.0,
    }
}

fn near(got: f64, want: f64, rel: f64, what: &str) {
    assert!(
        (got - want).abs() <= rel * want.abs(),
        "{what}: got {got}, the notes give {want}"
    );
}

#[test]
fn the_tank_barge_of_the_explanatory_notes() {
    let spaces = [tank("CO1", (20.0, 35.0)), tank("CO2", (35.0, 80.0))];
    let r = oil_outflow(&spaces, &barge()).unwrap();
    near(r.c_m3, 37_721.0, 1e-3, "C");
    near(r.rho_n, 900.0, 1e-3, "nominal density");
    let (co1, co2) = (&r.tanks[0], &r.tanks[1]);
    near(co1.oil_m3, 9_430.0, 1e-3, "CO1 at 98 %");
    near(co2.oil_m3, 28_291.0, 1e-3, "CO2 at 98 %");
    // 1.3.3: Ps; a symmetric barge has the same on both sides
    near(co1.ps_starboard, 0.0542, 5e-3, "Ps CO1");
    near(co2.ps_starboard, 0.1294, 5e-3, "Ps CO2");
    assert!((co1.ps_port - co1.ps_starboard).abs() < 1e-12);
    // 1.5.3: PB
    near(co1.pb_starboard, 0.0348, 5e-3, "PB CO1");
    near(co2.pb_starboard, 0.1452, 5e-3, "PB CO2");
    // 1.6.3: the outflow of each tank at 0.0 and -2.5 m tide
    near(co1.ob_0_m3, 5_471.0, 1e-3, "OB CO1 at 0 m");
    near(co2.ob_0_m3, 16_413.0, 1e-3, "OB CO2 at 0 m");
    near(co1.ob_25_m3, 6_993.0, 1e-3, "OB CO1 at -2.5 m");
    near(co2.ob_25_m3, 20_979.0, 1e-3, "OB CO2 at -2.5 m");
    assert_eq!(co1.cdb, 0.6);
    // 1.4.2, 1.6.6 to 1.7.1
    near(r.oms_m3, 4_172.0, 5e-3, "OMS");
    near(r.omb_0_m3, 1_544.0, 5e-3, "OMB(0)");
    near(r.omb_25_m3, 1_974.0, 5e-3, "OMB(2.5)");
    near(r.omb_m3, 1_673.0, 5e-3, "OMB");
    near(r.om, 0.0709, 5e-3, "OM");
    // 23.3.1: far over 0.015 for this barge, as the notes say it does not comply with all of MARPOL
    assert_eq!(r.limit, 0.015);
    assert!(r.ratio() > 4.0);
}

#[test]
fn the_cargo_level_follows_the_pressure_balance_and_a_tank_above_the_sea_loses_nothing_more_than_one_percent_at_the_bottom()
 {
    // A tank so high that the balance level is above its top retains all its oil: no outflow from a tank that
    // is not bounded by the bottom shell.
    let mut high = tank("H", (20.0, 35.0));
    high.shape = SpaceShape::Prism {
        section: vec![(-18.0, 2.0), (18.0, 2.0), (18.0, 3.0), (-18.0, 3.0)],
    };
    // the deadweight that makes the nominal density 900 kg/m3 for this tank: DW = 0.9 C
    let c = |t: &Space| 0.98 * 0.99 * (36.0 * (t.section()[2].1 - t.section()[0].1) * t.length());
    let inp = OilOutflowInputs {
        deadweight_t: 0.9 * c(&high),
        ..barge()
    };
    let r = oil_outflow(&[high], &inp).unwrap();
    assert_eq!(r.tanks[0].ob_0_m3, 0.0);
    // The same tank on the bottom shell loses at least 1 % of its oil (7.3.3).
    let mut low = tank("L", (20.0, 35.0));
    low.shape = SpaceShape::Prism {
        section: vec![(-18.0, 0.0), (18.0, 0.0), (18.0, 1.0), (-18.0, 1.0)],
    };
    let inp = OilOutflowInputs {
        deadweight_t: 0.9 * c(&low),
        ..barge()
    };
    let r = oil_outflow(&[low], &inp).unwrap();
    near(
        r.tanks[0].ob_0_m3,
        0.01 * r.tanks[0].oil_m3,
        1e-9,
        "1 % floor",
    );
    assert_eq!(r.tanks[0].cdb, 1.0);
}

#[test]
fn the_limit_of_23_3_1_falls_with_the_cargo_volume() {
    assert_eq!(om_limit(10_000.0), 0.015);
    assert_eq!(om_limit(200_000.0), 0.015);
    near(om_limit(300_000.0), 0.0135, 1e-12, "limit at 300,000 m3");
    assert_eq!(om_limit(400_000.0), 0.012);
    assert_eq!(om_limit(500_000.0), 0.012);
}

#[test]
fn no_cargo_volume_or_no_deadweight_gives_no_outflow() {
    assert!(oil_outflow(&[], &barge()).is_none());
    let spaces = [tank("CO1", (20.0, 35.0))];
    let mut inp = barge();
    inp.deadweight_t = 0.0;
    assert!(oil_outflow(&spaces, &inp).is_none());
}

// ---- 23.3.2: the longest cargo tank of a tanker of less than 5,000 t deadweight ----
// Label: sourced - MEPC.117(52) 23.3.2.1 to 23.3.2.3, worked by hand here for L = 90 m, B = 14 m, bi = 1.0 m
// (bi/B = 1/14).

use ovd_core::liquid_bulk::oil_outflow::max_cargo_tank_length_m;

fn wide(name: &str, x: (f64, f64), y: (f64, f64)) -> Space {
    Space {
        name: name.to_string(),
        kind: SpaceKind::CargoTank,
        side: Side::Centre,
        x_aft: x.0,
        x_fwd: x.1,
        shape: SpaceShape::Prism {
            section: vec![(y.0, 0.0), (y.1, 0.0), (y.1, 4.0), (y.0, 4.0)],
        },
    }
}

#[test]
fn the_longest_tank_follows_the_bulkheads_abreast_and_bi_over_b() {
    // one tank across (no longitudinal bulkhead): (0.5 / 14 + 0.1) 90 = 12.2143 m, under 0.2 L = 18 m
    let one = [wide("1", (10.0, 40.0), (-6.0, 6.0))];
    let m = max_cargo_tank_length_m(&one, 0, 90.0, 14.0).unwrap();
    assert!((m - (0.5 / 14.0 + 0.1) * 90.0).abs() < 1e-9, "{m}");
    // two across (a centre line bulkhead): (0.25 / 14 + 0.15) 90 = 15.1071 m
    let two = [
        wide("1P", (10.0, 40.0), (-6.0, 0.0)),
        wide("1S", (10.0, 40.0), (0.0, 6.0)),
    ];
    let m = max_cargo_tank_length_m(&two, 1, 90.0, 14.0).unwrap();
    assert!((m - (0.25 / 14.0 + 0.15) * 90.0).abs() < 1e-9, "{m}");
    // three across (two bulkheads): the wing tanks 0.2 L = 18 m; the centre tank, bi/B = 4 / 14 over 0.2, 18 m
    let three = [
        wide("1P", (10.0, 40.0), (-6.0, -2.0)),
        wide("1C", (10.0, 40.0), (-2.0, 2.0)),
        wide("1S", (10.0, 40.0), (2.0, 6.0)),
    ];
    assert_eq!(max_cargo_tank_length_m(&three, 0, 90.0, 14.0), Some(18.0));
    assert_eq!(max_cargo_tank_length_m(&three, 1, 90.0, 14.0), Some(18.0));
    // a centre tank with bi/B under 0.2: wing tanks 4 m wide in a 28 m beam, bi/B = 0.143: no centre line
    // bulkhead with three across: (0.5 * 4 / 28 + 0.1) 90 = 15.4286 m
    let narrow = [
        wide("1P", (10.0, 40.0), (-14.0, -10.0)),
        wide("1C", (10.0, 40.0), (-10.0, 10.0)),
        wide("1S", (10.0, 40.0), (10.0, 14.0)),
    ];
    let m = max_cargo_tank_length_m(&narrow, 1, 90.0, 28.0).unwrap();
    assert!((m - (0.5 * 4.0 / 28.0 + 0.1) * 90.0).abs() < 1e-9, "{m}");
    // a short hull: 10 m is the floor
    let short = [wide("1", (2.0, 8.0), (-1.0, 1.0))];
    assert_eq!(max_cargo_tank_length_m(&short, 0, 20.0, 2.0), Some(10.0));
    assert_eq!(max_cargo_tank_length_m(&short, 3, 20.0, 2.0), None);
}
