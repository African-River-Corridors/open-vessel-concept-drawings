//! Seam: `ovcd_core::loading_conditions`, the IS Code 2008 standard loading conditions (issue #426).
//! Label: sourced - IS Code 2008 (MSC.267(85)) Part B 3.4.1.2 (the four conditions and the 10 %
//! arrival consumables), 3.1.2 (98 % is nominally full) and 3.1.4 (consumables free surface), and
//! hand calculation (a box hull: T = Δ / (L B ρ), FSC = ρ i / Δ).

use ovcd_core::compartments::{Side, Space, SpaceKind, consumables_tanks};
use ovcd_core::gas_stability::PyError;
use ovcd_core::hydrostatics::{Hull, HullSpec};
use ovcd_core::loading_conditions::{
    BallastPlan, CargoAt, Consumables, Inputs, Item, Role, assess, roles,
};

const L: f64 = 80.0;
const B: f64 = 11.4;
const D: f64 = 4.0;

fn cargo_at(m: f64) -> Result<CargoAt, PyError> {
    Ok(CargoAt {
        item: Item {
            mass_t: m,
            kcg_m: 2.5,
            lcg_m: 40.0,
        },
        fill_fraction: 1.0,
        free_surface_moment_tm: 0.0,
    })
}

/// A double-bottom pair 1 m high, the length of the hull.
fn db_pair() -> Vec<Space> {
    [Side::Port, Side::Starboard]
        .into_iter()
        .map(|side| {
            let s = if side == Side::Port { -1.0 } else { 1.0 };
            let mut section = vec![
                (0.0, 0.0),
                (s * B / 2.0, 0.0),
                (s * B / 2.0, 1.0),
                (0.0, 1.0),
            ];
            if side == Side::Port {
                section.reverse();
            }
            Space {
                name: format!("DB1{}", if side == Side::Port { "P" } else { "S" }),
                kind: SpaceKind::DoubleBottomBallast,
                side,
                x_aft: 0.0,
                x_fwd: L,
                section,
            }
        })
        .collect()
}

fn inputs<'a>(
    lightship: &'a [Item],
    tanks: &'a [Space],
    plans: [BallastPlan; 2],
    consumables_t: f64,
) -> Inputs<'a> {
    Inputs {
        loa: L,
        beam: B,
        depth: D,
        hull: box_hull(),
        water_density: 1.0,
        criteria_set: "IS_CODE_2008",
        lightship,
        full_cargo_t: 1500.0,
        cargo_at: &cargo_at,
        consumables: Consumables {
            full_t: consumables_t,
            tanks: consumables_tanks(B, 2.0, consumables_t),
            density: 1.0,
        },
        ballast_tanks: tanks,
        ballast_density: 1.0,
        ballast_plans: plans,
        may_sail_without_ballast: false,
        air_draft_top_m: 8.0,
        air_draft_limit_m: None,
    }
}

/// The box L x B x D as its Sections Table (issue #448).
fn box_hull() -> &'static Hull {
    static HULL: std::sync::OnceLock<Hull> = std::sync::OnceLock::new();
    HULL.get_or_init(|| {
        Hull::build(&HullSpec {
            loa: L,
            beam: B,
            depth: D,
            bow_rake_length_m: 0.0,
            bow_rake_rise_m: D,
            stern: None,
            shape: Default::default(),
        })
        .unwrap()
    })
}

const LIGHT: [Item; 1] = [Item {
    mass_t: 600.0,
    kcg_m: 2.0,
    lcg_m: 40.0,
}];

#[test]
fn the_four_conditions_of_part_b_3_4_1_2_and_lightship_only_when_allowed() {
    assert_eq!(
        roles(false),
        [
            Role::FullLoadDeparture,
            Role::FullLoadArrival,
            Role::BallastDeparture,
            Role::BallastArrival
        ]
    );
    assert_eq!(roles(true).len(), 5);
    assert_eq!(roles(true)[4], Role::LightshipNoBallast);
}

#[test]
fn the_arrival_conditions_carry_ten_percent_of_the_consumables() {
    let tanks = db_pair();
    let inp = inputs(&LIGHT, &tanks, [BallastPlan::Auto, BallastPlan::Auto], 20.0);
    let c = assess(&inp).unwrap();
    assert!((c[0].consumables_t - 20.0).abs() < 1e-9);
    assert!((c[1].consumables_t - 2.0).abs() < 1e-9);
    assert!((c[2].consumables_t - 20.0).abs() < 1e-9);
    assert!((c[3].consumables_t - 2.0).abs() < 1e-9);
    // Displacement: lightship + cargo + consumables; T = Δ / (L B ρ) with Cb 1.
    assert!((c[1].condition.displacement_t - 2102.0).abs() < 1e-9);
    assert!((c[1].condition.draught_m - 2102.0 / (L * B)).abs() < 1e-12);
}

#[test]
fn the_full_load_conditions_sail_with_the_ballast_tanks_empty() {
    let tanks = db_pair();
    let fills = BallastPlan::Fills(vec![("DB1P".into(), 1.0), ("DB1S".into(), 1.0)]);
    let inp = inputs(&LIGHT, &tanks, [fills.clone(), fills], 0.0);
    let c = assess(&inp).unwrap();
    assert_eq!(c[0].ballast_t, 0.0);
    assert_eq!(c[1].ballast_t, 0.0);
    // Both 1 m double-bottom tanks pressed full: L × B × 1 m of water.
    assert!((c[2].ballast_t - L * B).abs() < 1e-6);
    assert_eq!(c[2].ballast.len(), 2);
}

#[test]
fn a_slack_ballast_tank_has_its_free_surface_and_a_nominally_full_one_has_none() {
    let tanks = db_pair();
    let half = BallastPlan::Fills(vec![("DB1P".into(), 0.5), ("DB1S".into(), 0.5)]);
    let full98 = BallastPlan::Fills(vec![("DB1P".into(), 0.98), ("DB1S".into(), 0.98)]);
    let c = assess(&inputs(&LIGHT, &tanks, [half, full98], 0.0)).unwrap();
    let dep = &c[2];
    // Two tanks, each I = L (B/2)³ / 12; FSC = ρ ΣI / Δ.
    let i = 2.0 * L * (B / 2.0) * (B / 2.0) * (B / 2.0) / 12.0;
    let disp = 600.0 + L * B * 0.5;
    assert!((dep.condition.free_surface_m - i / disp).abs() < 1e-6);
    assert!((dep.condition.gm_fluid_m - (dep.condition.gm_solid_m - i / disp)).abs() < 1e-9);
    assert_eq!(c[3].condition.free_surface_m, 0.0);
}

#[test]
fn the_consumables_have_their_largest_free_surface_in_every_condition() {
    let tanks = db_pair();
    let c = assess(&inputs(
        &LIGHT,
        &tanks,
        [BallastPlan::Auto, BallastPlan::Auto],
        20.0,
    ))
    .unwrap();
    // One transverse pair, each B/2 wide and 20 / (B × 1 m) long: I = l (B/2)³ / 12 each.
    let l = 20.0 / B;
    let i = 2.0 * l * (B / 2.0) * (B / 2.0) * (B / 2.0) / 12.0;
    let dep = &c[0].condition;
    assert!((dep.free_surface_m - i / dep.displacement_t).abs() < 1e-6);
    let arr = &c[1].condition;
    assert!((arr.free_surface_m - i / arr.displacement_t).abs() < 1e-6);
}

#[test]
fn the_auto_plan_finds_the_least_ballast_that_meets_every_criterion() {
    let tanks = db_pair();
    let mut inp = inputs(&LIGHT, &tanks, [BallastPlan::Auto, BallastPlan::Auto], 0.0);
    // Light draught 600 / (80 × 11.4) = 0.658 m, air draft 7.34 m. A limit of 7.0 m needs a
    // draught of 1.0 m: Δ 912 t, so at least 312 t of ballast, 34.2 % of the double bottom.
    inp.air_draft_limit_m = Some(7.0);
    let c = assess(&inp).unwrap();
    let dep = &c[2];
    assert_eq!(dep.plan_found, Some(true));
    assert!(dep.ok());
    let fill = dep.ballast[0].fill;
    assert!(fill >= 312.0 / (L * B));
    assert!((dep.ballast_t - fill * L * B).abs() < 1e-6);
    // One step less fails a criterion: the plan is the least in its steps.
    let less = BallastPlan::Fills(vec![
        ("DB1P".into(), fill - 0.05),
        ("DB1S".into(), fill - 0.05),
    ]);
    inp.ballast_plans = [less, BallastPlan::Auto];
    assert!(!assess(&inp).unwrap()[2].ok());
}

#[test]
fn no_plan_that_meets_every_criterion_fails_the_condition_and_says_so() {
    let tanks = db_pair();
    let mut inp = inputs(&LIGHT, &tanks, [BallastPlan::Auto, BallastPlan::Auto], 0.0);
    inp.air_draft_limit_m = Some(4.0);
    let c = assess(&inp).unwrap();
    assert_eq!(c[2].plan_found, Some(false));
    assert!(!c[2].ok());
    // Every ballast tank full.
    assert!(c[2].ballast.iter().all(|t| t.fill == 1.0));
}

#[test]
fn a_user_plan_is_used_as_given() {
    let tanks = db_pair();
    let plan = BallastPlan::Fills(vec![("DB1S".into(), 0.25), ("DB1P".into(), 0.25)]);
    let c = assess(&inputs(&LIGHT, &tanks, [plan, BallastPlan::Auto], 0.0)).unwrap();
    assert_eq!(c[2].plan_found, None);
    assert!((c[2].ballast_t - 0.25 * L * B).abs() < 1e-6);
}
