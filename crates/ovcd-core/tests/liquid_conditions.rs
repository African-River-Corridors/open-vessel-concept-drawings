//! Seam: the loading conditions of a liquid bulk hull (issue #332, spec 0006 sections 2 and 4).
//! Label: sourced - hand calculation (the free surface moment of a slack box tank, rho_cargo x l x
//! b^3 / 12, and its fall to a quarter with a centreline bulkhead; the displacement as the sum of the
//! weights, and KG as their mass-weighted mean) and IS Code 2008 (MSC.267(85)) Part B 3.1.2 (a tank
//! filled to 98 % or more is nominally full and has no free surface correction).

use ovcd_core::compartments::{Space, SpaceKind};
use ovcd_core::gas_stability::PyError;
use ovcd_core::hydrostatics::{Hull, HullSpec};
use ovcd_core::liquid_bulk::{LiquidBulkInputs, PumpArrangement, compute};
use ovcd_core::loading_conditions::{
    BallastPlan, Consumables, Inputs, Item, LiquidCargo, Role, assess, assess_slack,
};

const L: f64 = 80.0;
const B: f64 = 11.4;
const D: f64 = 3.6;
const RHO_CARGO: f64 = 0.87;

fn layout(centreline: bool) -> (Vec<Item>, LiquidCargo, Vec<Space>) {
    let r = compute(&LiquidBulkInputs {
        loa_m: L,
        beam_m: B,
        depth_m: D,
        zone_aft_m: 5.0,
        zone_fwd_m: 75.0,
        double_side_m: 1.0,
        double_bottom_m: 0.7,
        tanks_in_line: 3,
        centreline_bulkhead: centreline,
        cofferdam_m: 0.6,
        pump_arrangement: PumpArrangement::Deepwell,
        slop_share: 0.0,
        fill_ratio: 0.98,
        cargo_density_t_m3: RHO_CARGO,
        heating_kw_per_m3: 0.0,
        lightship_k_t_per_m3: 0.1141,
    })
    .unwrap();
    let items = r.lightship_items.iter().map(|(_, i)| *i).collect();
    let tanks = r
        .spaces
        .iter()
        .filter(|s| s.kind == SpaceKind::CargoTank)
        .cloned()
        .collect();
    let ballast = r
        .spaces
        .iter()
        .filter(|s| s.kind.is_ballast())
        .cloned()
        .collect();
    (
        items,
        LiquidCargo {
            tanks,
            density: RHO_CARGO,
        },
        ballast,
    )
}

fn cargo_fn(
    c: &LiquidCargo,
) -> impl Fn(f64) -> Result<ovcd_core::loading_conditions::CargoAt, PyError> + '_ {
    move |m| c.at(m)
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

fn conditions(
    centreline: bool,
    cargo_fraction: f64,
    slack: &[f64],
) -> (
    Vec<ovcd_core::loading_conditions::LoadedCondition>,
    LiquidCargo,
) {
    let (lightship, cargo, ballast) = layout(centreline);
    let at = cargo_fn(&cargo);
    let inp = Inputs {
        loa: L,
        beam: B,
        depth: D,
        hull: box_hull(),
        water_density: 1.0,
        criteria_set: "IS_CODE_2008",
        lightship: &lightship,
        full_cargo_t: cargo.mass_at_fill(cargo_fraction),
        cargo_at: &at,
        consumables: Consumables {
            full_t: 0.0,
            tanks: Vec::new(),
            density: 1.0,
        },
        ballast_tanks: &ballast,
        ballast_density: 1.0,
        ballast_plans: [BallastPlan::Auto, BallastPlan::Auto],
        may_sail_without_ballast: false,
        air_draft_top_m: 6.0,
        air_draft_limit_m: None,
    };
    let mut conds = assess(&inp).unwrap();
    conds.extend(assess_slack(&inp, slack).unwrap());
    (conds, cargo.clone())
}

fn near(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-9 * (1.0 + b.abs()), "{a} != {b}");
}

// Inside width 11.4 - 2 x 1.0 = 9.4 m. Three tanks of (70 - 1.2) / 3 m. No centreline bulkhead:
// each tank is 9.4 m wide, i = l x 9.4^3 / 12. With one: 4.7 m wide, twice as many tanks.
#[test]
fn the_free_surface_of_the_cargo_is_density_l_b_cubed_over_12_per_slack_tank() {
    let l = (70.0 - 1.2) / 3.0;
    let (_, one, _) = layout(false);
    let c = one.at(one.mass_at_fill(0.5)).unwrap();
    near(
        c.free_surface_moment_tm,
        RHO_CARGO * 3.0 * l * (9.4 * 9.4 * 9.4) / 12.0,
    );
    near(c.fill_fraction, 0.5);
    let (_, two, _) = layout(true);
    let c2 = two.at(two.mass_at_fill(0.5)).unwrap();
    near(
        c2.free_surface_moment_tm,
        RHO_CARGO * 6.0 * l * (4.7 * 4.7 * 4.7) / 12.0,
    );
    // A centreline bulkhead cuts the free surface to a quarter.
    near(c2.free_surface_moment_tm, c.free_surface_moment_tm / 4.0);
}

#[test]
fn a_tank_at_the_nominally_full_level_has_no_free_surface() {
    let (_, cargo, _) = layout(false);
    for f in [0.98, 0.99, 1.0] {
        assert_eq!(
            cargo
                .at(cargo.mass_at_fill(f))
                .unwrap()
                .free_surface_moment_tm,
            0.0,
            "{f}"
        );
    }
    assert!(
        cargo
            .at(cargo.mass_at_fill(0.97))
            .unwrap()
            .free_surface_moment_tm
            > 0.0
    );
    // No cargo, no free surface.
    assert_eq!(cargo.at(0.0).unwrap().free_surface_moment_tm, 0.0);
}

#[test]
fn the_cargo_centre_is_the_tank_centre_at_every_fill() {
    let (_, cargo, _) = layout(true);
    // Rectangular tanks 0.7 m to 3.6 m: at a fill x the liquid is 2.9 m x deep.
    for x in [0.1, 0.5, 0.9] {
        let c = cargo.at(cargo.mass_at_fill(x)).unwrap();
        near(c.item.kcg_m, 0.7 + 2.9 * x / 2.0);
        near(c.item.mass_t, cargo.mass_at_fill(x));
    }
}

#[test]
fn a_laden_condition_at_the_volume_bound_is_full_a_weight_bound_one_is_slack() {
    let (full, cargo) = conditions(true, 0.98, &[]);
    assert_eq!(full[0].condition.free_surface_m, 0.0);
    let (slack, _) = conditions(true, 0.6, &[]);
    let laden = &slack[0];
    assert_eq!(laden.role, Role::FullLoadDeparture);
    let l = (70.0 - 1.2) / 3.0;
    let moment = RHO_CARGO * 6.0 * l * (4.7 * 4.7 * 4.7) / 12.0;
    // Free surface correction = moment / displacement (no consumables, no ballast).
    near(
        laden.condition.free_surface_m,
        moment / laden.condition.displacement_t,
    );
    near(laden.condition.cargo_t, cargo.mass_at_fill(0.6));
    assert!(laden.condition.gm_fluid_m < laden.condition.gm_solid_m);
}

#[test]
fn every_condition_lists_its_weights_and_they_add_up() {
    let (conds, _) = conditions(true, 0.7, &[]);
    for c in &conds {
        let mass: f64 = c.weights.iter().map(|(_, i)| i.mass_t).sum();
        near(mass, c.condition.displacement_t);
        let moment: f64 = c.weights.iter().map(|(_, i)| i.mass_t * i.kcg_m).sum();
        near(moment / mass, c.condition.kg_m);
        assert_eq!(c.weights[0].0, "Lightship");
    }
    let laden = &conds[0];
    let names: Vec<&str> = laden.weights.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["Lightship", "Cargo"]);
    // A ballast condition lists the ballast tanks it fills, by name.
    let ballast = conds
        .iter()
        .find(|c| c.role == Role::BallastDeparture)
        .unwrap();
    assert!(
        ballast
            .weights
            .iter()
            .any(|(n, _)| n.starts_with("Ballast DB")),
        "{:?}",
        ballast.weights
    );
    assert!(ballast.ballast_t > 0.0);
    assert!(ballast.weights.iter().all(|(n, _)| n != "Cargo"));
}

#[test]
fn the_slack_condition_is_the_candidate_with_the_lowest_fluid_gm() {
    let (_, cargo, _) = layout(true);
    let candidates = cargo.slack_masses_t();
    assert_eq!(candidates.len(), 19);
    assert!(candidates.iter().all(|m| cargo.fill_of(*m).unwrap() < 0.98));
    let (conds, _) = conditions(true, 0.98, &candidates);
    let slack = conds.last().unwrap();
    assert_eq!(slack.role, Role::SlackCargo);
    // Each candidate alone gives its own fluid GM: the chosen one is the least of them.
    let alone: Vec<f64> = candidates
        .iter()
        .map(|m| {
            conditions(true, 0.98, &[*m])
                .0
                .last()
                .unwrap()
                .condition
                .gm_fluid_m
        })
        .collect();
    let least = alone.iter().copied().fold(f64::INFINITY, f64::min);
    near(slack.condition.gm_fluid_m, least);
    assert!(
        candidates
            .iter()
            .any(|m| (m - slack.condition.cargo_t).abs() < 1e-9)
    );
    // It is a slack condition: the free surface is there.
    assert!(slack.condition.free_surface_m > 0.0);
    // And it carries the departure consumables fraction, with no ballast.
    assert_eq!(slack.ballast_t, 0.0);
}

#[test]
fn with_no_candidate_there_is_no_slack_condition() {
    let (conds, _) = conditions(false, 0.98, &[]);
    assert!(conds.iter().all(|c| c.role != Role::SlackCargo));
    assert_eq!(conds.len(), 4);
}

#[test]
fn the_slack_condition_is_never_one_of_the_standard_roles() {
    assert!(!Role::ALL.contains(&Role::SlackCargo));
    assert_eq!(Role::SlackCargo.key(), "slack_cargo");
    assert_eq!(Role::SlackCargo.name(), "Slack cargo tanks, worst case");
}
