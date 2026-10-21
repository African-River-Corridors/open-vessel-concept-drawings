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
//! The hull is a prismatic box, as in the gas stability: KB = T/2, BM = B²/12T, and the GZ curve of
//! [`crate::box_stability`].

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::box_stability::{Condition, PyError, box_condition_at, box_gm_solid, criterion, div};
use crate::compartments::{Space, SpaceKind};
use crate::hull::HullGeometry;
use crate::models::TrimResult;
use crate::stern_form::SternGeometry;

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
        }
    }

    /// The part of the full stores and fuel the condition carries.
    pub fn consumables_fraction(self) -> f64 {
        match self {
            Role::FullLoadDeparture | Role::BallastDeparture => 1.0,
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
    /// The tanks that hold them, filled in step. Empty: no consumables.
    pub tanks: Vec<Space>,
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

/// The inputs of [`assess`].
pub struct Inputs<'a> {
    pub loa: f64,
    pub beam: f64,
    pub depth: f64,
    pub block_coefficient: f64,
    pub water_density: f64,
    pub criteria_set: &'a str,
    pub lightship: &'a [Item],
    /// The cargo of the fully loaded conditions (t).
    pub full_cargo_t: f64,
    /// Where a cargo mass sits, and its free surface.
    pub cargo_at: &'a dyn Fn(f64) -> Result<CargoAt, PyError>,
    pub consumables: Consumables,
    pub ballast_tanks: &'a [Space],
    pub ballast_density: f64,
    /// The plan of each ballast condition: departure, then arrival.
    pub ballast_plans: [BallastPlan; 2],
    pub may_sail_without_ballast: bool,
    /// The highest fixed point above the keel (m), for the air draft.
    pub air_draft_top_m: f64,
    pub air_draft_limit_m: Option<f64>,
    pub stern: Option<&'a SternGeometry>,
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
}

impl LoadedCondition {
    /// Every criterion met: the stability, and the air draft limit when there is one.
    pub fn ok(&self) -> bool {
        self.condition.ok() && self.air_draft_ok != Some(false)
    }
}

/// The tank groups of the automatic plan, in fill order: the double bottom, then the peaks, then
/// the wing tanks. The tanks of a group fill together, so the fill is symmetric.
pub fn auto_groups(tanks: &[Space]) -> Vec<Vec<usize>> {
    [
        SpaceKind::DoubleBottomBallast,
        SpaceKind::PeakBallast,
        SpaceKind::WingBallast,
    ]
    .into_iter()
    .map(|k| {
        (0..tanks.len())
            .filter(|&i| tanks[i].kind == k)
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
                                .find(|(n, _)| *n == t.name)
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
    let mut c = condition(inp, role, &fills)?;
    if c.ok() {
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
            c = condition(inp, role, &fills)?;
            if c.ok() {
                c.plan_found = Some(true);
                return Ok(c);
            }
        }
    }
    c.plan_found = Some(false);
    Ok(c)
}

fn filled(space: &Space, fill: f64, density: f64) -> TankFill {
    let f = space.fill(fill);
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
    let volume: f64 = c.tanks.iter().map(Space::volume_m3).sum();
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
    let (l, b, d) = (inp.loa, inp.beam, inp.depth);
    let mut items: Vec<Item> = inp.lightship.to_vec();
    let mut fs_moment = 0.0;
    let cargo_t = if role.carries_cargo() {
        inp.full_cargo_t
    } else {
        0.0
    };
    let mut cargo_fill = 0.0;
    if cargo_t > 0.0 {
        let c = (inp.cargo_at)(cargo_t)?;
        items.push(c.item);
        fs_moment += c.free_surface_moment_tm;
        cargo_fill = c.fill_fraction;
    }
    let (cons, cons_fs) = consumables_at(&inp.consumables, role.consumables_fraction());
    let consumables_t = cons.iter().fold(0.0, |a, i| a + i.mass_t);
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
        items.push(Item {
            mass_t: t.mass_t,
            kcg_m: t.vcg_m,
            lcg_m: t.lcg_m,
        });
        fs_moment += t.free_surface_moment_tm;
    }
    let ballast_t = ballast.iter().fold(0.0, |a, t| a + t.mass_t);

    let disp: f64 = items.iter().map(|i| i.mass_t).sum();
    let kg = div(items.iter().map(|i| i.mass_t * i.kcg_m).sum(), disp)?;
    let t = div(disp, inp.block_coefficient * l * b * inp.water_density)?;
    let hull = HullGeometry::new(l, b, d, 0.0, 0.0, 0.0, 0.0);
    let along: Vec<(f64, f64)> = items.iter().map(|i| (i.mass_t, i.lcg_m)).collect();
    let trim = crate::hydrostatics::trim_with_stern(&hull, t, &along, inp.stern);
    let air_draft = inp.air_draft_top_m - t;

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
            criteria: alloc::vec![criterion(
                "Floats with freeboard",
                t,
                d,
                false,
                "draught \u{2265} depth",
            )],
        }
    } else {
        let gm_solid = box_gm_solid(b, t, kg)?;
        box_condition_at(
            b,
            d,
            t,
            kg,
            gm_solid,
            div(fs_moment, disp)?,
            inp.criteria_set,
        )?
    };
    c.name = role.name().to_string();
    c.cargo_t = cargo_t;
    c.fill_fraction = cargo_fill;
    c.displacement_t = disp;
    Ok(LoadedCondition {
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
    })
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
    let tank_bottom = g.double_bottom_height + g.tank_support_height;
    let r_in = g.inner_diameter / 2.0;
    let zone_centre = (g.stern_void_m + (l - g.bow_void_m)) / 2.0;
    let mut items = alloc::vec![
        Item {
            mass_t: g.steel_top_down,
            kcg_m: 0.5 * d,
            lcg_m: l / 2.0
        },
        Item {
            mass_t: g.tank_total + g.insulation,
            kcg_m: tank_bottom + g.outer_diameter / 2.0,
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
