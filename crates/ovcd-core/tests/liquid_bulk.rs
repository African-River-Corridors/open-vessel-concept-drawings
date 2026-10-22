//! Label: sourced - hand calculation: the volume of a box, a box's centroid and the free surface
//! inertia of a rectangle, i = l b^3 / 12, worked out by hand for the dimensions in each test.

use ovcd_core::compartments::{Side, Space, SpaceKind};
use ovcd_core::liquid_bulk::{
    LiquidBulkInputs, LiquidBulkResult, PumpArrangement, compute, lightship,
};

fn base() -> LiquidBulkInputs {
    LiquidBulkInputs {
        loa_m: 60.0,
        beam_m: 10.0,
        depth_m: 4.0,
        zone_aft_m: 5.0,
        zone_fwd_m: 55.0,
        double_side_m: 1.0,
        double_bottom_m: 0.7,
        tanks_in_line: 2,
        centreline_bulkhead: false,
        cofferdam_m: 0.6,
        pump_arrangement: PumpArrangement::Deepwell,
        slop_share: 0.0,
        fill_ratio: 0.98,
        cargo_density_t_m3: 0.87,
        heating_kw_per_m3: 0.0,
        lightship_k_t_per_m3: lightship::TANK_LIGHTSHIP_K_T_PER_M3,
    }
}

fn near(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-9 * (1.0 + b.abs()), "{a} != {b}");
}

fn kind(r: &LiquidBulkResult, k: SpaceKind) -> Vec<&Space> {
    r.of_kind(k).collect()
}

// Zone 50 m. Block = 50 - 2 x 0.6 = 48.8 m, so two tanks of 24.4 m. Inner breadth 10 - 2 x 1.0 = 8 m.
// Inner height 4 - 0.7 = 3.3 m. One tank = 24.4 x 8 x 3.3 = 644.16 m3.
#[test]
fn box_tank_volume_and_centroid() {
    let r = compute(&base()).unwrap();
    let tanks = kind(&r, SpaceKind::CargoTank);
    assert_eq!(tanks.len(), 2);
    assert_eq!(tanks[0].name, "1C");
    near(tanks[0].volume_m3(), 644.16);
    near(r.cargo_gross_m3, 1288.32);
    near(r.cargo_volume_m3, 1288.32 * 0.98);
    near(r.cargo_mass_t, 1288.32 * 0.98 * 0.87);
    // Tank 1: from 5.0 + 0.6 = 5.6 m to 30.0 m: the centroid is at 17.8 m, on the centre line, at
    // 0.7 + 3.3 / 2 = 2.35 m.
    let f = tanks[0].fill(1.0);
    near(f.lcg_m, 17.8);
    near(f.tcg_m, 0.0);
    near(f.vcg_m, 2.35);
    near(f.volume_m3, 644.16);
}

// A tank of l = 24.4 m and b = 8 m: i = 24.4 x 512 / 12.
#[test]
fn free_surface_of_a_slack_box_tank_is_l_b_cubed_over_12() {
    let r = compute(&base()).unwrap();
    assert_eq!(r.free_surface.len(), 2);
    for fs in &r.free_surface {
        near(fs.length_m, 24.4);
        near(fs.breadth_m, 8.0);
        near(fs.inertia_m4, 24.4 * 512.0 / 12.0);
    }
}

// A centreline bulkhead halves each breadth, so i of a tank is an eighth: l x 4^3 / 12. There are
// twice as many tanks, so the total free surface inertia is a quarter: 2 x 64 / 512.
#[test]
fn a_centreline_bulkhead_cuts_the_free_surface_to_a_quarter() {
    let one = compute(&base()).unwrap();
    let mut inp = base();
    inp.centreline_bulkhead = true;
    let two = compute(&inp).unwrap();
    let tanks = kind(&two, SpaceKind::CargoTank);
    assert_eq!(tanks.len(), 4);
    assert_eq!(
        tanks.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
        ["1P", "1S", "2P", "2S"]
    );
    assert_eq!(tanks[0].side, Side::Port);
    assert_eq!(tanks[1].side, Side::Starboard);
    for fs in &two.free_surface {
        near(fs.breadth_m, 4.0);
        near(fs.inertia_m4, 24.4 * 64.0 / 12.0);
    }
    let sum = |r: &LiquidBulkResult| r.free_surface.iter().map(|f| f.inertia_m4).sum::<f64>();
    near(sum(&two), sum(&one) / 4.0);
    near(two.cargo_gross_m3, one.cargo_gross_m3);
    // The centroid of a port tank is to port.
    assert!(tanks[0].fill(1.0).tcg_m < 0.0 && tanks[1].fill(1.0).tcg_m > 0.0);
}

#[test]
fn the_spaces_add_up_to_the_cargo_zone() {
    let r = compute(&base()).unwrap();
    let total: f64 = r.spaces.iter().map(|s| s.volume_m3()).sum();
    near(total, 10.0 * 4.0 * 50.0);
    // Double bottom: the full beam, 0.7 m high, along the 48.8 m block.
    let db: f64 = kind(&r, SpaceKind::DoubleBottomBallast)
        .iter()
        .map(|s| s.volume_m3())
        .sum();
    near(db, 10.0 * 0.7 * 48.8);
    let wb: f64 = kind(&r, SpaceKind::WingBallast)
        .iter()
        .map(|s| s.volume_m3())
        .sum();
    near(wb, 2.0 * 1.0 * 3.3 * 48.8);
    let cd: f64 = kind(&r, SpaceKind::Cofferdam)
        .iter()
        .map(|s| s.volume_m3())
        .sum();
    near(cd, 2.0 * 0.6 * 10.0 * 4.0);
}

#[test]
fn a_slop_tank_takes_its_share_of_the_tank_block() {
    let mut inp = base();
    inp.slop_share = 0.03;
    let r = compute(&inp).unwrap();
    let slop = kind(&r, SpaceKind::Slop);
    assert_eq!(slop.len(), 1);
    assert_eq!(slop[0].name, "SL1C");
    // 3 % of the 48.8 m block, aft of the cargo tanks.
    near(slop[0].length(), 0.03 * 48.8);
    near(slop[0].x_aft, 5.6);
    near(r.slop_share_actual, 0.03);
    near(r.slop_gross_m3 + r.cargo_gross_m3, 8.0 * 3.3 * 48.8);
    // The slop tank is not cargo: it is not in the capacity.
    let plain = compute(&base()).unwrap();
    assert!(r.cargo_gross_m3 < plain.cargo_gross_m3);
}

#[test]
fn a_pump_room_takes_the_aft_end_and_is_not_cargo() {
    let mut inp = base();
    inp.pump_arrangement = PumpArrangement::PumpRoom { length_m: 4.0 };
    let r = compute(&inp).unwrap();
    let pr = kind(&r, SpaceKind::PumpRoom);
    assert_eq!(pr.len(), 1);
    near(pr[0].x_aft, 5.0);
    near(pr[0].length(), 4.0);
    // The block is now 50 - 4 - 1.2 = 44.8 m.
    near(r.cargo_gross_m3, 8.0 * 3.3 * 44.8);
    // With deepwell pumps there is no pump room.
    assert!(kind(&compute(&base()).unwrap(), SpaceKind::PumpRoom).is_empty());
}

fn extent(s: &Space) -> ([f64; 2], [f64; 2], [f64; 2]) {
    let fold = |f: fn(&(f64, f64)) -> f64| {
        s.section
            .iter()
            .map(f)
            .fold([f64::INFINITY, f64::NEG_INFINITY], |[lo, hi], v| {
                [lo.min(v), hi.max(v)]
            })
    };
    ([s.x_aft, s.x_fwd], fold(|p| p.0), fold(|p| p.1))
}

fn overlap(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[1].min(b[1]) - a[0].max(b[0])).max(0.0)
}

// Property: for every layout, no two Spaces overlap and together they fill the cargo zone exactly.
#[test]
fn spaces_never_overlap_and_fill_the_cargo_zone() {
    let mut cases = 0;
    for beam in [8.0, 10.7, 16.5, 22.8] {
        for tanks in [1_u32, 2, 3, 6] {
            for centreline in [false, true] {
                for slop in [0.0, 0.03, 0.05] {
                    for pump in [
                        PumpArrangement::Deepwell,
                        PumpArrangement::PumpRoom { length_m: 5.0 },
                    ] {
                        for (side, bottom) in [(0.0, 0.0), (0.76, 0.76), (1.0, 0.7)] {
                            let inp = LiquidBulkInputs {
                                loa_m: 100.0,
                                beam_m: beam,
                                depth_m: 5.0,
                                zone_aft_m: 8.0,
                                zone_fwd_m: 92.0,
                                double_side_m: side,
                                double_bottom_m: bottom,
                                tanks_in_line: tanks,
                                centreline_bulkhead: centreline,
                                cofferdam_m: 0.6,
                                pump_arrangement: pump,
                                slop_share: slop,
                                ..base()
                            };
                            let r = compute(&inp).unwrap();
                            let total: f64 = r.spaces.iter().map(|s| s.volume_m3()).sum();
                            near(total, beam * 5.0 * 84.0);
                            for (i, a) in r.spaces.iter().enumerate() {
                                for b in &r.spaces[i + 1..] {
                                    let (ax, ay, az) = extent(a);
                                    let (bx, by, bz) = extent(b);
                                    let v = overlap(ax, bx) * overlap(ay, by) * overlap(az, bz);
                                    assert!(
                                        v < 1e-9,
                                        "{} and {} overlap by {v} m3",
                                        a.name,
                                        b.name
                                    );
                                }
                            }
                            cases += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cases, 4 * 4 * 2 * 3 * 2 * 3);
}

#[test]
fn the_tanks_are_numbered_from_aft_and_the_order_is_fixed() {
    let mut inp = base();
    inp.centreline_bulkhead = true;
    inp.slop_share = 0.03;
    inp.pump_arrangement = PumpArrangement::PumpRoom { length_m: 3.0 };
    let r = compute(&inp).unwrap();
    let names: Vec<&str> = r.spaces.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "1P", "1S", "2P", "2S", "SL1P", "SL1S", "DB1P", "DB1S", "WB1P", "WB1S", "CD1", "CD2",
            "PR1"
        ]
    );
    let t = kind(&r, SpaceKind::CargoTank);
    assert!(t[0].x_fwd <= t[2].x_aft + 1e-12);
}

#[test]
fn a_hull_with_no_double_side_has_no_wing_ballast() {
    let mut inp = base();
    inp.double_side_m = 0.0;
    let r = compute(&inp).unwrap();
    assert!(kind(&r, SpaceKind::WingBallast).is_empty());
    near(
        kind(&r, SpaceKind::CargoTank)[0].volume_m3(),
        24.4 * 10.0 * 3.3,
    );
}

#[test]
fn bad_inputs_are_refused_with_a_clear_message() {
    type Edit = fn(&mut LiquidBulkInputs);
    let bad: [(Edit, &str); 9] = [
        (|i| i.beam_m = 0.0, "beam_m must be a positive number"),
        (|i| i.tanks_in_line = 0, "tanks_in_line must be at least 1"),
        (
            |i| i.slop_share = 1.0,
            "slop_share must be 0 or more and under 1",
        ),
        (
            |i| i.fill_ratio = 0.0,
            "fill_ratio must be over 0 and at most 1",
        ),
        (
            |i| i.double_side_m = 5.0,
            "the double side must be less than half the beam",
        ),
        (
            |i| i.double_bottom_m = 4.0,
            "the double bottom must be lower than the depth",
        ),
        (
            |i| i.zone_fwd_m = 61.0,
            "the cargo zone must lie inside the hull",
        ),
        (|i| i.cofferdam_m = -1.0, "cofferdam_m must not be negative"),
        (|i| i.tanks_in_line = 60, "too short for"),
    ];
    for (edit, text) in bad {
        let mut inp = base();
        edit(&mut inp);
        let e = compute(&inp).unwrap_err();
        assert_eq!(e.kind, "ValueError");
        assert!(e.message.contains(text), "{}: {}", text, e.message);
    }
    let mut inp = base();
    inp.beam_m = f64::NAN;
    assert!(compute(&inp).is_err());
}

// Lightship: the hull is k x L x B x D; the outfit is the volume times the coefficients.
#[test]
fn the_lightship_is_the_hull_plus_the_outfit() {
    let mut inp = base();
    inp.heating_kw_per_m3 = 0.3;
    let r = compute(&inp).unwrap();
    let hull = inp.lightship_k_t_per_m3 * 60.0 * 10.0 * 4.0;
    let tanks = r.cargo_gross_m3 + r.slop_gross_m3;
    let outfit = lightship::outfit_t(tanks, 0.3 * tanks);
    near(r.lightship_t, hull + outfit);
    let names: Vec<&str> = r.lightship_items.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        ["hull", "pumps", "piping", "vents and gauging", "heating"]
    );
    near(
        r.lightship_items.iter().map(|(_, i)| i.mass_t).sum::<f64>(),
        r.lightship_t,
    );
    // No heating, no heating item.
    let plain = compute(&base()).unwrap();
    assert!(plain.lightship_items.iter().all(|(n, _)| n != "heating"));
    near(plain.lightship_items[0].1.kcg_m, 2.0);
}

// The constants equal the generated fit: calibration/tank_lightship.yaml (tools/fit_tank_lightship.py).
#[test]
fn the_lightship_constants_equal_the_generated_fit() {
    let text = include_str!("../../../calibration/tank_lightship.yaml");
    let value_after = |key: &str| -> f64 {
        let at = text
            .find(key)
            .unwrap_or_else(|| panic!("{key} not in the fit"));
        let rest = &text[at..];
        let v = rest.find("value:").unwrap();
        rest[v + 6..]
            .lines()
            .next()
            .unwrap()
            .trim()
            .parse()
            .unwrap()
    };
    near(
        value_after("tank_lightship_k_t_per_m3:"),
        lightship::TANK_LIGHTSHIP_K_T_PER_M3,
    );
    near(value_after("pumps_t_per_m3:"), lightship::PUMPS_T_PER_M3);
    near(value_after("piping_t_per_m3:"), lightship::PIPING_T_PER_M3);
    near(
        value_after("vents_gauging_t_per_m3:"),
        lightship::VENTS_GAUGING_T_PER_M3,
    );
    near(
        value_after("heating_t_per_kw:"),
        lightship::HEATING_T_PER_KW,
    );
}
