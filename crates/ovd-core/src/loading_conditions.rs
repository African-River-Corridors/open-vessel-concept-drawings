//! The IS Code standard loading conditions of a cargo ship, with consumables and water ballast
//! (issue #426).
//!
//! IMO IS Code 2008 (MSC.267(85)) Part B 3.4.1.2 lists four conditions: fully loaded departure with
//! full stores and fuel, fully loaded arrival with 10 % stores and fuel, in ballast departure with
//! full stores and fuel, and in ballast arrival with 10 %. Part A 2.1.1 applies every criterion to
//! each of them, and 2.1.2 the free surface effects of Part B 3.1. Part B 3.4.2.2 has the fully
//! loaded departure with the water ballast tanks empty; 3.4.2.3 asks that the quantity and the
//! disposition of any water ballast be stated. "Lightship, no ballast" is added for a vessel that
//! may sail that way; lightship has no consumables, stores or ballast (Introduction 2.23).
//!
//! The hull is its Sections Table (issue #448), as in the gas stability: each condition floats at
//! free trim, its GM is KM_T of the upright cut less KG, and its GZ curve is the sections' one,
//! judged by [`crate::box_stability`].

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::box_stability::{
    Condition, PyError, criterion, div, failed_condition, hull_condition, trim_out_of_range,
};
use crate::compartments::{ClippedSpace, Space, SpaceKind};
use crate::hydrostatics::Hull;
use crate::models::TrimResult;

/// IS Code 2008 Part B 3.1.2: a tank filled to 98 % or more is nominally full and has no free
/// surface correction.
pub const NOMINALLY_FULL: f64 = 0.98;

/// IS Code 2008 Part B 3.4.1.2: the arrival conditions carry 10 % of the stores and fuel.
pub const ARRIVAL_CONSUMABLES: f64 = 0.10;

/// The step of the automatic ballast plan's search, as a fraction of a tank group's volume.
pub const AUTO_BALLAST_STEP: f64 = 0.05;

/// The role of a loading condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Part B 3.4.1.2.1.
    FullLoadDeparture,
    /// Part B 3.4.1.2.2.
    FullLoadArrival,
    /// Part B 3.4.1.2.3.
    BallastDeparture,
    /// Part B 3.4.1.2.4.
    BallastArrival,
    /// No cargo, no consumables, no ballast: only for a vessel that may sail that way.
    LightshipNoBallast,
    /// Liquid bulk (spec 0006): the departure condition with the cargo tanks slack at the fill
    /// whose fluid GM is lowest. Not one of [`Role::ALL`]: [`assess_slack`] makes it.
    SlackCargo,
}

impl Role {
    pub const ALL: [Role; 5] = [
        Role::FullLoadDeparture,
        Role::FullLoadArrival,
        Role::BallastDeparture,
        Role::BallastArrival,
        Role::LightshipNoBallast,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Role::FullLoadDeparture => "full_load_departure",
            Role::FullLoadArrival => "full_load_arrival",
            Role::BallastDeparture => "ballast_departure",
            Role::BallastArrival => "ballast_arrival",
            Role::LightshipNoBallast => "lightship_no_ballast",
            Role::SlackCargo => "slack_cargo",
        }
    }

    /// The condition's name in the results.
    pub fn name(self) -> &'static str {
        match self {
            Role::FullLoadDeparture => "Full load, departure",
            Role::FullLoadArrival => "Full load, arrival",
            Role::BallastDeparture => "Ballast, departure",
            Role::BallastArrival => "Ballast, arrival",
            Role::LightshipNoBallast => "Lightship, no ballast",
            Role::SlackCargo => "Slack cargo tanks, worst case",
        }
    }

    /// The part of the full stores and fuel the condition carries.
    pub fn consumables_fraction(self) -> f64 {
        match self {
            Role::FullLoadDeparture | Role::BallastDeparture | Role::SlackCargo => 1.0,
            Role::FullLoadArrival | Role::BallastArrival => ARRIVAL_CONSUMABLES,
            Role::LightshipNoBallast => 0.0,
        }
    }

    pub fn carries_cargo(self) -> bool {
        matches!(self, Role::FullLoadDeparture | Role::FullLoadArrival)
    }

    /// Whether the condition takes a ballast plan. The full load conditions sail with the ballast
    /// tanks empty (Part B 3.4.2.2).
    pub fn takes_ballast(self) -> bool {
        matches!(self, Role::BallastDeparture | Role::BallastArrival)
    }
}

/// A mass at a point: (t, KG m above the keel, LCG m from the aft end).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Item {
    pub mass_t: f64,
    pub kcg_m: f64,
    pub lcg_m: f64,
}

/// The cargo at one mass: where it sits and its free surface moment (t·m: density × inertia).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CargoAt {
    pub item: Item,
    /// The part of the cargo space the cargo fills, 0 to 1.
    pub fill_fraction: f64,
    pub free_surface_moment_tm: f64,
}

/// The stores and fuel: their tanks, filled in proportion to the condition's fraction.
#[derive(Debug, Clone, PartialEq)]
pub struct Consumables {
    /// The full mass (t).
    pub full_t: f64,
    /// The tanks that hold them, as the hull clips them (issue #631), filled in step. Empty: no
    /// consumables.
    pub tanks: Vec<ClippedSpace>,
    /// The density the tanks are filled with (t/m³).
    pub density: f64,
}

/// How the ballast tanks of a ballast condition are filled.
#[derive(Debug, Clone, PartialEq)]
pub enum BallastPlan {
    /// The least ballast, in the fill order of [`auto_groups`], that meets every criterion.
    Auto,
    /// A fill per tank, by name, 0 to 1. A tank not named is empty.
    Fills(Vec<(String, f64)>),
}

/// A user ballast plan by tank group, both sides alike: the fill of the double bottom, the peaks
/// and the wing tanks, 0 to 1.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GroupFills {
    pub double_bottom: f64,
    pub peak: f64,
    pub wing: f64,
}

impl GroupFills {
    /// The plan as a fill per tank of `tanks`, by name.
    pub fn plan(&self, tanks: &[ClippedSpace]) -> BallastPlan {
        BallastPlan::Fills(
            tanks
                .iter()
                .map(|t| {
                    let f = match t.space.kind {
                        SpaceKind::DoubleBottomBallast => self.double_bottom,
                        SpaceKind::PeakBallast => self.peak,
                        SpaceKind::WingBallast => self.wing,
                        // Not ballast: a ballast plan never fills them.
                        SpaceKind::Consumables
                        | SpaceKind::CargoTank
                        | SpaceKind::Slop
                        | SpaceKind::Cofferdam
                        | SpaceKind::PumpRoom
                        | SpaceKind::CargoHold
                        | SpaceKind::Void
                        | SpaceKind::Machinery
                        | SpaceKind::PeakVoid => 0.0,
                    };
                    (t.space.name.clone(), f)
                })
                .collect(),
        )
    }
}

/// The inputs of [`assess`].
pub struct Inputs<'a> {
    pub loa: f64,
    pub beam: f64,
    pub depth: f64,
    /// Issue #448: the hull's sections (the raked bow and the Stern Form) give each condition's
    /// draught, trim and GZ.
    pub hull: &'a Hull,
    pub water_density: f64,
    pub criteria_set: &'a str,
    pub lightship: &'a [Item],
    /// The cargo of the fully loaded conditions (t).
    pub full_cargo_t: f64,
    /// Where a cargo mass sits, and its free surface.
    pub cargo_at: &'a dyn Fn(f64) -> Result<CargoAt, PyError>,
    pub consumables: Consumables,
    /// The ballast tanks, clipped by the hull (issue #455).
    pub ballast_tanks: &'a [ClippedSpace],
    pub ballast_density: f64,
    /// The plan of each ballast condition: departure, then arrival.
    pub ballast_plans: [BallastPlan; 2],
    pub may_sail_without_ballast: bool,
    /// The highest fixed point above the keel (m), for the air draft.
    pub air_draft_top_m: f64,
    pub air_draft_limit_m: Option<f64>,
}

/// One tank's water ballast in a condition.
#[derive(Debug, Clone, PartialEq)]
pub struct TankFill {
    pub name: String,
    pub kind: SpaceKind,
    pub fill: f64,
    pub mass_t: f64,
    pub lcg_m: f64,
    pub tcg_m: f64,
    pub vcg_m: f64,
    /// Density × free-surface inertia (t·m); zero when nominally full or empty.
    pub free_surface_moment_tm: f64,
}

/// One loading condition, assessed.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedCondition {
    pub role: Role,
    /// The stability: name, cargo, displacement, draught, KG, GM, GZ and the criteria.
    pub condition: Condition,
    pub consumables_t: f64,
    pub ballast: Vec<TankFill>,
    pub ballast_t: f64,
    pub trim: TrimResult,
    pub freeboard_m: f64,
    pub air_draft_m: f64,
    /// Whether the air draft is within the limit; `None` with no limit.
    pub air_draft_ok: Option<bool>,
    /// For a ballast condition with an automatic plan: whether a plan meets every criterion.
    /// `None` for a user plan or no ballast.
    pub plan_found: Option<bool>,
    /// The weights of the condition and where they are: the lightship, the cargo, the stores and
    /// fuel, and each ballast tank, as (name, mass, KG, LCG). Their sum is the displacement.
    pub weights: Vec<(String, Item)>,
}

impl LoadedCondition {
    /// Every criterion met: the stability, and the air draft limit when there is one.
    pub fn ok(&self) -> bool {
        self.condition.ok() && self.air_draft_ok != Some(false)
    }
}

/// The tank groups of the automatic plan, in fill order: the double bottom, then the peaks, then
/// the wing tanks. The tanks of a group fill together, so the fill is symmetric.
pub fn auto_groups(tanks: &[ClippedSpace]) -> Vec<Vec<usize>> {
    [
        SpaceKind::DoubleBottomBallast,
        SpaceKind::PeakBallast,
        SpaceKind::WingBallast,
    ]
    .into_iter()
    .map(|k| {
        (0..tanks.len())
            .filter(|&i| tanks[i].space.kind == k)
            .collect::<Vec<_>>()
    })
    .filter(|g| !g.is_empty())
    .collect()
}

/// The conditions that apply: the four of Part B 3.4.1.2, and "Lightship, no ballast" when the
/// vessel may sail that way.
pub fn roles(may_sail_without_ballast: bool) -> Vec<Role> {
    Role::ALL
        .into_iter()
        .filter(|r| *r != Role::LightshipNoBallast || may_sail_without_ballast)
        .collect()
}

/// Assess every condition that applies.
pub fn assess(inp: &Inputs) -> Result<Vec<LoadedCondition>, PyError> {
    let mut out = Vec::new();
    for role in roles(inp.may_sail_without_ballast) {
        let c = if role.takes_ballast() {
            let plan = match role {
                Role::BallastDeparture => &inp.ballast_plans[0],
                _ => &inp.ballast_plans[1],
            };
            match plan {
                BallastPlan::Fills(fills) => {
                    let f: Vec<f64> = inp
                        .ballast_tanks
                        .iter()
                        .map(|t| {
                            fills
                                .iter()
                                .find(|(n, _)| *n == t.space.name)
                                .map_or(0.0, |(_, f)| f.clamp(0.0, 1.0))
                        })
                        .collect();
                    condition(inp, role, &f)?
                }
                BallastPlan::Auto => auto_plan(inp, role)?,
            }
        } else {
            condition(inp, role, &alloc::vec![0.0; inp.ballast_tanks.len()])?
        };
        out.push(c);
    }
    Ok(out)
}

/// The least ballast, in the fill order of [`auto_groups`], in steps of [`AUTO_BALLAST_STEP`], that
/// meets every criterion. If none does, the plan with every ballast tank full, marked not found.
fn auto_plan(inp: &Inputs, role: Role) -> Result<LoadedCondition, PyError> {
    let mut fills = alloc::vec![0.0; inp.ballast_tanks.len()];
    // Issue #448: a trial that fails a criterion known before the GZ curve (the air draft, the
    // fluid GM, the freeboard) is not ok whatever its curve, so it is screened out without one.
    // The trial found, or with none the last one, is worked in full, so no reported number differs.
    if let Some(mut c) = trial(inp, role, &fills)? {
        c.plan_found = Some(true);
        return Ok(c);
    }
    let steps = libm::round(1.0 / AUTO_BALLAST_STEP) as usize;
    for group in auto_groups(inp.ballast_tanks) {
        for k in 1..=steps {
            let f = (k as f64 * AUTO_BALLAST_STEP).min(1.0);
            for &i in &group {
                fills[i] = f;
            }
            if let Some(mut c) = trial(inp, role, &fills)? {
                c.plan_found = Some(true);
                return Ok(c);
            }
        }
    }
    let mut c = condition(inp, role, &fills)?;
    c.plan_found = Some(false);
    Ok(c)
}

/// One trial of the automatic plan: the condition when it is ok, `None` when it is not.
fn trial(inp: &Inputs, role: Role, fills: &[f64]) -> Result<Option<LoadedCondition>, PyError> {
    let c = match condition_with(inp, role, fills, None, true)? {
        Some(c) => c,
        None => return Ok(None),
    };
    Ok(if c.ok() { Some(c) } else { None })
}

fn filled(tank: &ClippedSpace, fill: f64, density: f64) -> TankFill {
    let (f, space) = (tank.fill(fill), &tank.space);
    let fs = if fill >= NOMINALLY_FULL {
        0.0
    } else {
        density * f.free_surface_inertia_m4
    };
    TankFill {
        name: space.name.clone(),
        kind: space.kind,
        fill: f.fraction,
        mass_t: f.volume_m3 * density,
        lcg_m: f.lcg_m,
        tcg_m: f.tcg_m,
        vcg_m: f.vcg_m,
        free_surface_moment_tm: fs,
    }
}

/// The consumables' items at `fraction` of full, and their free surface moment. Part B 3.1.3.2
/// and 3.1.4: consumable liquids are variable fills, so the correction is the largest the tanks
/// reach between empty and full, whatever the fill; every consumables tank counts.
fn consumables_at(c: &Consumables, fraction: f64) -> (Vec<Item>, f64) {
    if c.full_t <= 0.0 || fraction <= 0.0 || c.tanks.is_empty() {
        return (Vec::new(), 0.0);
    }
    let volume: f64 = c.tanks.iter().map(|t| t.volume_m3).sum();
    let tank_fill = if volume > 0.0 {
        (c.full_t * fraction / (c.density * volume)).min(1.0)
    } else {
        0.0
    };
    let items = c
        .tanks
        .iter()
        .map(|t| {
            let f = t.fill(tank_fill);
            Item {
                mass_t: f.volume_m3 * c.density,
                kcg_m: f.vcg_m,
                lcg_m: f.lcg_m,
            }
        })
        .collect();
    let fs = c
        .tanks
        .iter()
        .map(|t| c.density * t.fill(0.5).free_surface_inertia_m4)
        .sum();
    (items, fs)
}

/// One condition at the given ballast fills (one per ballast tank).
fn condition(inp: &Inputs, role: Role, fills: &[f64]) -> Result<LoadedCondition, PyError> {
    Ok(condition_with(inp, role, fills, None, false)?
        .expect("an unscreened condition is always worked"))
}

/// The mass of `items` as one item: its total, and the mass-weighted KG and LCG.
fn lumped(items: &[Item]) -> Item {
    let m: f64 = items.iter().fold(0.0, |a, i| a + i.mass_t);
    if m == 0.0 {
        return Item {
            mass_t: 0.0,
            kcg_m: 0.0,
            lcg_m: 0.0,
        };
    }
    Item {
        mass_t: m,
        kcg_m: items.iter().fold(0.0, |a, i| a + i.mass_t * i.kcg_m) / m,
        lcg_m: items.iter().fold(0.0, |a, i| a + i.mass_t * i.lcg_m) / m,
    }
}

/// The fluid GM below which the IS Code 2008 Part A 2.2.4 criterion fails (m).
const GM_FLUID_MIN_M: f64 = 0.15;

/// [`condition`], with the cargo mass given (the slack cargo condition takes a part load). With
/// `screen`, `None` for a condition that fails a criterion known before its GZ
/// curve (issue #448): the air draft, the fluid GM, or the freeboard (the hull does not float).
fn condition_with(
    inp: &Inputs,
    role: Role,
    fills: &[f64],
    cargo_override: Option<f64>,
    screen: bool,
) -> Result<Option<LoadedCondition>, PyError> {
    let d = inp.depth;
    let mut items: Vec<Item> = inp.lightship.to_vec();
    let mut weights: Vec<(String, Item)> = Vec::new();
    weights.push(("Lightship".to_string(), lumped(&items)));
    let mut fs_moment = 0.0;
    let cargo_t = match cargo_override {
        Some(m) => m,
        None if role.carries_cargo() => inp.full_cargo_t,
        None => 0.0,
    };
    let mut cargo_fill = 0.0;
    if cargo_t > 0.0 {
        let c = (inp.cargo_at)(cargo_t)?;
        weights.push(("Cargo".to_string(), c.item));
        items.push(c.item);
        fs_moment += c.free_surface_moment_tm;
        cargo_fill = c.fill_fraction;
    }
    let (cons, cons_fs) = consumables_at(&inp.consumables, role.consumables_fraction());
    let consumables_t = cons.iter().fold(0.0, |a, i| a + i.mass_t);
    if consumables_t > 0.0 {
        weights.push(("Stores and fuel".to_string(), lumped(&cons)));
    }
    items.extend(cons);
    fs_moment += cons_fs;
    let ballast: Vec<TankFill> = inp
        .ballast_tanks
        .iter()
        .zip(fills)
        .filter(|&(_, &f)| f > 0.0)
        .map(|(t, &f)| filled(t, f, inp.ballast_density))
        .collect();
    for t in &ballast {
        let item = Item {
            mass_t: t.mass_t,
            kcg_m: t.vcg_m,
            lcg_m: t.lcg_m,
        };
        weights.push((alloc::format!("Ballast {}", t.name), item));
        items.push(item);
        fs_moment += t.free_surface_moment_tm;
    }
    let ballast_t = ballast.iter().fold(0.0, |a, t| a + t.mass_t);

    let disp: f64 = items.iter().map(|i| i.mass_t).sum();
    let kg = div(items.iter().map(|i| i.mass_t * i.kcg_m).sum(), disp)?;
    let lcg = div(items.iter().map(|i| i.mass_t * i.lcg_m).sum(), disp)?;
    let g = crate::sections::Gravity {
        lcg_m: lcg,
        tcg_m: 0.0,
        kg_m: kg,
    };
    if inp.water_density == 0.0 {
        return Err(PyError::ZeroDivision);
    }
    // Issue #448: the free-trim float of the sections; the draught at mid-length. A load the whole
    // hull cannot float has no draught: it fails "Floats with freeboard".
    let to_py = |e: crate::sections::SectionsError| PyError::Value(e.to_string());
    let sinks = disp / inp.water_density >= inp.hull.volume_at(d);
    // The equilibrium at heel 0, which the GZ curve starts from.
    let mut heel0 = None;
    let (t, trim) = if sinks {
        (
            d,
            TrimResult {
                LCG: lcg,
                LCB: lcg,
                trim: 0.0,
                draft_aft: d,
                draft_fwd: d,
            },
        )
    } else {
        let f = match inp.hull.float(disp, g, inp.water_density) {
            Ok(f) => f,
            // No balance within the trim solver's range: the condition fails at its level draught.
            Err(crate::sections::SectionsError::TrimOutOfRange) => {
                let t = inp
                    .hull
                    .level_draught(disp, inp.water_density)
                    .map_err(to_py)?;
                let mut c = failed_condition(t, kg, trim_out_of_range());
                c.name = role.name().to_string();
                c.cargo_t = cargo_t;
                c.fill_fraction = cargo_fill;
                c.displacement_t = disp;
                let air_draft = inp.air_draft_top_m - t;
                return Ok(Some(LoadedCondition {
                    role,
                    condition: c,
                    consumables_t,
                    ballast,
                    ballast_t,
                    trim: TrimResult {
                        LCG: lcg,
                        LCB: lcg,
                        trim: 0.0,
                        draft_aft: t,
                        draft_fwd: t,
                    },
                    freeboard_m: d - t,
                    air_draft_m: air_draft,
                    air_draft_ok: inp.air_draft_limit_m.map(|limit| air_draft <= limit),
                    plan_found: None,
                    weights,
                }));
            }
            Err(e) => return Err(to_py(e)),
        };
        let r = (
            f.draught_mid_m,
            TrimResult {
                LCG: lcg,
                LCB: f.equilibrium.immersed.lcb_m,
                trim: f.trim_m,
                draft_aft: f.draught_aft_m,
                draft_fwd: f.draught_fwd_m,
            },
        );
        heel0 = Some(f.equilibrium);
        r
    };
    let air_draft = inp.air_draft_top_m - t;

    let air_draft_ok = inp.air_draft_limit_m.map(|limit| air_draft <= limit);
    if screen && (t >= d || air_draft_ok == Some(false)) {
        return Ok(None);
    }
    if screen && t < d {
        let gm_solid = inp
            .hull
            .upright(t, inp.water_density)
            .map_err(to_py)?
            .km_t_m
            - kg;
        if gm_solid - div(fs_moment, disp)? < GM_FLUID_MIN_M {
            return Ok(None);
        }
    }
    let mut c = if t >= d {
        Condition {
            name: String::new(),
            cargo_t: 0.0,
            cargo_is_int: false,
            fill_fraction: 0.0,
            displacement_t: 0.0,
            draught_m: t,
            kg_m: kg,
            gm_solid_m: -1.0,
            free_surface_m: 0.0,
            gm_fluid_m: -1.0,
            gz: Vec::new(),
            max_gz_m: 0.0,
            max_gz_deg: 0,
            area_0_27: 0.0,
            area_0_30: 0.0,
            area_0_40: 0.0,
            area_30_40: 0.0,
            deck_edge_deg: None,
            downflooding_deg: None,
            criteria: alloc::vec![criterion(
                "Floats with freeboard",
                t,
                d,
                false,
                "draught \u{2265} depth",
            )],
        }
    } else {
        let gm_solid = inp
            .hull
            .upright(t, inp.water_density)
            .map_err(to_py)?
            .km_t_m
            - kg;
        let fs_m = div(fs_moment, disp)?;
        // Issue #567: a trial of the automatic plan stops as soon as a criterion of the IS Code
        // set is decided (see `screened_gz`); the curve of a trial that is not decided is the
        // whole curve, used as is.
        // No screen with openings listed and no angle typed: the angle of down-flooding is worked
        // from the heels' waterplanes (issue #592) and can end the areas below 40 degrees.
        let angle_from_openings =
            inp.hull.spec.downflooding_deg.is_none() && !inp.hull.spec.openings.is_empty();
        let gz_worked = if screen && inp.criteria_set == "IS_CODE_2008" && !angle_from_openings {
            match crate::box_stability::screened_gz(
                inp.hull,
                disp,
                g,
                fs_m * disp,
                inp.water_density,
                heel0.as_ref(),
            )? {
                crate::box_stability::Screened::Fails => return Ok(None),
                crate::box_stability::Screened::Curve(gz) => Some(gz),
            }
        } else {
            None
        };
        hull_condition(
            inp.hull,
            disp,
            g,
            t,
            gm_solid,
            fs_m,
            inp.water_density,
            inp.criteria_set,
            heel0.as_ref(),
            gz_worked,
        )?
    };
    c.name = role.name().to_string();
    c.cargo_t = cargo_t;
    c.fill_fraction = cargo_fill;
    c.displacement_t = disp;
    Ok(Some(LoadedCondition {
        role,
        condition: c,
        consumables_t,
        ballast,
        ballast_t,
        trim,
        freeboard_m: d - t,
        air_draft_m: air_draft,
        air_draft_ok: inp.air_draft_limit_m.map(|limit| air_draft <= limit),
        plan_found: None,
        weights,
    }))
}

/// The worst slack-tank condition of a liquid bulk hull (spec 0006 section 2, row 5): the departure
/// condition (full stores and fuel, ballast tanks empty) with the cargo tanks all filled alike to a
/// part load, at the cargo mass in `candidates_t` whose fluid GM is the lowest. A box tank's free
/// surface inertia is the same at every slack level, so what moves the fluid GM is the weight and
/// the centre of the cargo. A mass that fills the tanks to the nominally full level has no free
/// surface and is not a slack case: the caller leaves it out. `None` with no candidate.
pub fn assess_slack(
    inp: &Inputs,
    candidates_t: &[f64],
) -> Result<Option<LoadedCondition>, PyError> {
    let none = alloc::vec![0.0; inp.ballast_tanks.len()];
    let mut worst: Option<LoadedCondition> = None;
    for &m in candidates_t {
        let c = condition_with(inp, Role::SlackCargo, &none, Some(m), false)?
            .expect("an unscreened condition is always worked");
        // The first of equal fluid GM stays, so the result does not depend on rounding.
        if worst
            .as_ref()
            .is_none_or(|w| c.condition.gm_fluid_m < w.condition.gm_fluid_m)
        {
            worst = Some(c);
        }
    }
    Ok(worst)
}

/// The cargo of a gas barge: liquid in the Type C tanks, the same model as the gas stability
/// (`gas_stability::assess`): a horizontal cylinder segment, its free surface along every tank's
/// cylinder.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GasCargo {
    pub density: f64,
    pub inner_radius: f64,
    /// The inner bottom of the tanks above the keel (m).
    pub inner_bottom_m: f64,
    pub total_volume_m3: f64,
    pub n_tanks: f64,
    pub cylinder_length_m: f64,
    /// The middle of the tank zone, from the aft end (m).
    pub lcg_m: f64,
}

impl GasCargo {
    pub fn at(&self, mass_t: f64) -> Result<CargoAt, PyError> {
        let frac = if self.total_volume_m3 > 0.0 {
            div(mass_t, self.density * self.total_volume_m3)?
        } else {
            0.0
        };
        let r = self.inner_radius;
        let (h, ybar) = crate::gas_stability::segment_fill(r, frac)?;
        let i_fs = if 0.0 < frac && frac < 1.0 {
            let chord = 2.0 * libm::sqrt((r * r - (r - h) * (r - h)).max(0.0));
            self.n_tanks * self.cylinder_length_m * chord * chord * chord / 12.0
        } else {
            0.0
        };
        Ok(CargoAt {
            item: Item {
                mass_t,
                kcg_m: self.inner_bottom_m + ybar,
                lcg_m: self.lcg_m,
            },
            fill_fraction: frac,
            free_surface_moment_tm: self.density * i_fs,
        })
    }
}

/// The lightship items and the cargo model of a gas barge, with the heights and positions the gas
/// stability gives them (`gas_stability::assess`, issues #348 and #349). `None` for a dry result.
pub fn gas_model(
    result: &crate::gas_stability::GasCapacityResult,
) -> Option<(Vec<Item>, GasCargo)> {
    let g = result.gas_basis.as_ref()?;
    let (l, d) = (g.loa, g.depth);
    // The tanks' height and their place along the hull, from the Tank Layout (spec 0009).
    let tank_bottom = crate::tanks::tank_bottom(g.double_bottom_height, g.tank_support_height);
    let r_in = g.inner_diameter / 2.0;
    let zone_centre = crate::tanks::CargoZone::new(l, g.bow_void_m, g.stern_void_m).centre();
    let mut items = alloc::vec![
        Item {
            mass_t: g.hull_steel,
            kcg_m: 0.5 * d,
            lcg_m: l / 2.0
        },
        Item {
            mass_t: g.tank_total + g.insulation,
            kcg_m: crate::tanks::tank_axis_z(tank_bottom, g.outer_diameter),
            lcg_m: zone_centre,
        },
        Item {
            mass_t: g.piping_machinery,
            kcg_m: d,
            lcg_m: l / 2.0
        },
        Item {
            mass_t: g.outfitting,
            kcg_m: 0.75 * d,
            lcg_m: g.stern_void_m / 2.0
        },
    ];
    // Issue #410: the covering above D at its own centres.
    items.extend(g.covering_items.iter().map(|i| Item {
        mass_t: i.mass_t,
        kcg_m: i.kcg_m,
        lcg_m: i.lcg_m,
    }));
    items.extend(result.stern_block_items.iter().map(|i| Item {
        mass_t: i.mass_t,
        kcg_m: i.kcg_m,
        lcg_m: i.lcg_m,
    }));
    let cargo = GasCargo {
        density: result.cargo_density_t_per_m3,
        inner_radius: r_in,
        inner_bottom_m: tank_bottom + (g.outer_diameter / 2.0 - r_in),
        total_volume_m3: g.total_tank_volume,
        n_tanks: g.n_tanks_total as f64,
        cylinder_length_m: g.cylinder_length,
        lcg_m: zone_centre,
    };
    Some((items, cargo))
}

/// The lightship items and the cargo of a dry bulk hull, as the hold GM has them
/// (`barge_capacity::bulk_gm`): the lightship at half the depth and mid-length, the stern block
/// items at their own points, and the cargo at the centroid of the full trapezoidal hold, in the
/// middle of the hold zone. Solid bulk has no free surface.
pub fn dry_model(
    inp: &crate::barge_capacity::BargeCapacityInputs,
    r: &crate::barge_capacity::BargeCapacityResult,
) -> (Vec<Item>, Item) {
    let block: f64 = r.stern_block_items.iter().map(|i| i.mass_t).sum();
    let mut items = alloc::vec![Item {
        mass_t: r.lightship_t - block,
        kcg_m: 0.5 * inp.depth,
        lcg_m: inp.loa / 2.0,
    }];
    items.extend(r.stern_block_items.iter().map(|i| Item {
        mass_t: i.mass_t,
        kcg_m: i.kcg_m,
        lcg_m: i.lcg_m,
    }));
    let w_top = r.hold_top_width_m.unwrap_or(0.0);
    let w_bot = r.hold_bottom_width_m.unwrap_or(0.0);
    let h = r.hold_height_m.unwrap_or(0.0);
    let zc = if w_top + w_bot > 0.0 {
        (h / 3.0) * (2.0 * w_top + w_bot) / (w_top + w_bot)
    } else {
        h / 2.0
    };
    let stern = r.stern_void_length_m.unwrap_or(0.0);
    let bow = r.bow_void_length_m.unwrap_or(0.0);
    let cargo = Item {
        mass_t: 0.0,
        kcg_m: inp.double_bottom_height_bulk + zc,
        lcg_m: (stern + (inp.loa - bow)) / 2.0,
    };
    (items, cargo)
}

/// The cargo of a liquid bulk hull: liquid in the rectangular cargo tanks of the `liquid_bulk`
/// module, every tank filled alike. The slop tanks stay empty. A tank at the nominally full level
/// (Part B 3.1.2) has no free surface; a slack tank has the free surface of its section.
#[derive(Debug, Clone, PartialEq)]
pub struct LiquidCargo {
    /// The cargo tanks, in the order of the layout.
    pub tanks: Vec<Space>,
    /// The cargo density (t/m³).
    pub density: f64,
}

impl LiquidCargo {
    /// The gross volume of the cargo tanks (m³).
    pub fn gross_m3(&self) -> f64 {
        self.tanks.iter().fold(0.0, |a, t| a + t.volume_m3())
    }

    /// The cargo mass of a fill fraction of the gross volume (t).
    pub fn mass_at_fill(&self, fraction: f64) -> f64 {
        fraction * self.gross_m3() * self.density
    }

    /// The fill fraction of the gross volume that holds `mass_t`, at most 1.
    pub fn fill_of(&self, mass_t: f64) -> Result<f64, PyError> {
        let capacity = self.gross_m3() * self.density;
        if capacity <= 0.0 {
            return Ok(0.0);
        }
        Ok(div(mass_t, capacity)?.clamp(0.0, 1.0))
    }

    /// The cargo at `mass_t`: its centre and its free surface moment.
    pub fn at(&self, mass_t: f64) -> Result<CargoAt, PyError> {
        let x = self.fill_of(mass_t)?;
        let fills: Vec<(f64, f64, f64, f64, f64)> = self
            .tanks
            .iter()
            .map(|t| {
                let f = t.fill(x);
                (
                    f.volume_m3 * self.density,
                    f.vcg_m,
                    f.lcg_m,
                    f.free_surface_inertia_m4,
                    f.fraction,
                )
            })
            .collect();
        let mass: f64 = fills.iter().fold(0.0, |a, f| a + f.0);
        let (kcg, lcg) = if mass > 0.0 {
            (
                fills.iter().fold(0.0, |a, f| a + f.0 * f.1) / mass,
                fills.iter().fold(0.0, |a, f| a + f.0 * f.2) / mass,
            )
        } else {
            (0.0, 0.0)
        };
        let fs = if x >= NOMINALLY_FULL {
            0.0
        } else {
            self.density * fills.iter().fold(0.0, |a, f| a + f.3)
        };
        Ok(CargoAt {
            item: Item {
                mass_t: mass,
                kcg_m: kcg,
                lcg_m: lcg,
            },
            fill_fraction: x,
            free_surface_moment_tm: fs,
        })
    }

    /// The part loads the worst slack condition is searched over: 5 % to 95 % of the gross volume
    /// in steps of 5 %. All are below the nominally full level.
    pub fn slack_masses_t(&self) -> Vec<f64> {
        (1..=19)
            .map(|k| self.mass_at_fill(f64::from(k) * 0.05))
            .collect()
    }
}

/// The lightship items and the cargo model of a liquid bulk hull: the items of the `liquid_bulk`
/// layout (the hull and the outfit, each at its own point) and the stern block items. `None` for a
/// gas or dry bulk result.
pub fn liquid_model(
    r: &crate::barge_capacity::BargeCapacityResult,
) -> Option<(Vec<Item>, LiquidCargo)> {
    let l = r.liquid.as_ref()?;
    let mut items: Vec<Item> = l.lightship_items.iter().map(|(_, i)| *i).collect();
    items.extend(r.stern_block_items.iter().map(|i| Item {
        mass_t: i.mass_t,
        kcg_m: i.kcg_m,
        lcg_m: i.lcg_m,
    }));
    let tanks = l
        .spaces
        .iter()
        .filter(|s| s.kind == SpaceKind::CargoTank)
        .cloned()
        .collect();
    Some((
        items,
        LiquidCargo {
            tanks,
            density: r.cargo_density_t_per_m3,
        },
    ))
}
