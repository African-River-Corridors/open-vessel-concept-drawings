//! The liquid bulk (crude oil) path of `simulate`: the tank layout read from the params, and the
//! `liquid` block of the results (issue #331, spec 0006). The capacity and the stability chain is the
//! one of `ovcd_core::barge_capacity::compute`, with the liquid layout (`ovcd_core::liquid_bulk`) as its
//! volume branch. Nothing here runs for a gas or a dry bulk Cargo, so their results keep every key.

use crate::eval::num;
use ovcd_core::barge_capacity::BargeCapacityResult;
use ovcd_core::compartments::{Space, SpaceKind};
use ovcd_core::liquid_bulk::adn_vessel_type::{self, AdnFlags};
use ovcd_core::liquid_bulk::crude_grade;
use ovcd_core::liquid_bulk::{LiquidBulkResult, LiquidLayout, PumpArrangement};
use serde_json::{Map, Value, json};

fn number(p: &Map<String, Value>, key: &str) -> Result<f64, String> {
    p.get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("`params.{key}` must be a number"))
}

/// The cargo tank layout of the params: the inputs of the `integral_tanks` containment. Every key is
/// required: the Defaults Profile gives each one a default, so a Design always has them.
pub(crate) fn layout_of(p: &Map<String, Value>) -> Result<LiquidLayout, String> {
    let tanks = number(p, "liquid_tanks_in_line")?;
    if tanks.fract() != 0.0 || !(1.0..=1e6).contains(&tanks) {
        return Err("`params.liquid_tanks_in_line` must be a whole number of at least 1".into());
    }
    let pump_arrangement = match p.get("pump_arrangement").and_then(Value::as_str) {
        Some("deepwell") => PumpArrangement::Deepwell,
        Some("pump_room") => PumpArrangement::PumpRoom {
            length_m: number(p, "pump_room_length_m")?,
        },
        _ => return Err("`params.pump_arrangement` must be `deepwell` or `pump_room`".into()),
    };
    let centreline_bulkhead = p
        .get("centreline_bulkhead")
        .and_then(Value::as_bool)
        .ok_or("`params.centreline_bulkhead` must be true or false")?;
    Ok(LiquidLayout {
        double_side_m: number(p, "double_side_m")?,
        double_bottom_m: number(p, "double_bottom_m")?,
        tanks_in_line: tanks as u32,
        centreline_bulkhead,
        cofferdam_m: number(p, "cofferdam_m")?,
        pump_arrangement,
        slop_share: number(p, "slop_share")?,
        fill_ratio: number(p, "liquid_fill_ratio")?,
        heating_kw_per_m3: number(p, "heating_kw_per_m3")?,
        lightship_k_t_per_m3: number(p, "lightship_k_t_per_m3")?,
    })
}

fn space_json(s: &Space) -> Value {
    let full = s.fill(1.0);
    json!({
        "name": s.name,
        "kind": s.kind.key(),
        "side": s.side.key(),
        "x_aft_m": num(s.x_aft),
        "x_fwd_m": num(s.x_fwd),
        "volume_m3": num(s.volume_m3()),
        "lcg_m": num(full.lcg_m),
        "tcg_m": num(full.tcg_m),
        "kcg_m": num(full.vcg_m),
    })
}

/// The ADN Tank Vessel Type of the Cargo, from its Crude Grade, with the branch that decided it.
fn adn_type_json(cargo_key: &str) -> Value {
    match crude_grade::grade_of(cargo_key) {
        Some(g) => {
            let r = adn_vessel_type::derive(&AdnFlags::of(g));
            json!({
                "type": r.vessel_type.key(),
                "walls_distinct_from_hull": r.walls_distinct_from_hull,
                "basis": r.branch.text(),
                "grade": g.name,
            })
        }
        None => Value::Null,
    }
}

/// The `liquid` block of the results: the Spaces, the volumes, the free surfaces, the lightship
/// items and the ADN Tank Vessel Type. The masses and volumes are what the cost lines read.
pub(crate) fn liquid_json(l: &LiquidBulkResult, cargo_key: &str) -> Value {
    json!({
        "tanks_in_line": l.tanks_in_line,
        "cargo_tank_count": l.of_kind(SpaceKind::CargoTank).count(),
        "cargo_gross_m3": num(l.cargo_gross_m3),
        "slop_gross_m3": num(l.slop_gross_m3),
        "cargo_volume_m3": num(l.cargo_volume_m3),
        "cargo_volume_bound_t": num(l.cargo_mass_t),
        "slop_share": num(l.slop_share_actual),
        "cargo_tank_length_min_m": num(l.cargo_tank_length_min_m),
        "cargo_tank_length_max_m": num(l.cargo_tank_length_max_m),
        "cargo_tank_breadth_max_m": num(l.cargo_tank_breadth_max_m),
        "adn_vessel_type": adn_type_json(cargo_key),
        "spaces": l.spaces.iter().map(space_json).collect::<Vec<_>>(),
        "free_surface": l.free_surface.iter().map(|f| json!({
            "name": f.name,
            "length_m": num(f.length_m),
            "breadth_m": num(f.breadth_m),
            "inertia_m4": num(f.inertia_m4),
        })).collect::<Vec<_>>(),
        "lightship_items": l.lightship_items.iter().map(|(name, i)| json!({
            "name": name,
            "mass_t": num(i.mass_t),
            "kcg_m": num(i.kcg_m),
            "lcg_m": num(i.lcg_m),
        })).collect::<Vec<_>>(),
    })
}

/// A Space for the drawing: the Space as the results give it, with its cross-section corners
/// (`section_m`, (y, z) in metres), which the general arrangement draws.
fn drawing_space_json(s: &Space) -> Value {
    let mut v = space_json(s);
    v["section_m"] = Value::Array(
        s.section
            .iter()
            .map(|(y, z)| json!([num(*y), num(*z)]))
            .collect(),
    );
    v
}

/// The ballast tanks of a dry bulk hull for the drawing, as `drawing.ballast_spaces` (issue #426).
/// Kept out of the results hash with the rest of `drawing`.
pub(crate) fn ballast_drawing_value(tanks: &[Space]) -> (&'static str, Value) {
    (
        "ballast_spaces",
        Value::Array(tanks.iter().map(drawing_space_json).collect()),
    )
}

/// The `drawing` values of a liquid Option: the layout the general arrangement will draw. Kept out
/// of the results hash with the rest of `drawing`.
pub(crate) fn drawing_values(r: &BargeCapacityResult) -> Vec<(&'static str, Value)> {
    let Some(l) = r.liquid.as_ref() else {
        return Vec::new();
    };
    vec![(
        "liquid_tanks",
        json!({
            "spaces": l.spaces.iter().map(drawing_space_json).collect::<Vec<_>>(),
        }),
    )]
}

/// Issue #341: the assumed damages of the hull and what each does to the laden departure condition.
///
/// The first list is by the ADN extents of the vessel type of the Cargo (9.3.2.15.1 for type C, 9.3.3.15.1 for
/// type N); it is empty when the Cargo has no ADN type. The second is by the extents of MARPOL Annex I
/// regulation 28.2, worked out only when `marpol` is true (the Rule Set is in the stack). Both are empty when the
/// departure condition has no floating position.
pub(crate) fn damage_verdicts(
    hull: &ovcd_core::hydrostatics::Hull,
    liquid: &LiquidBulkResult,
    cargo_key: &str,
    density: f64,
    conditions: &[ovcd_core::loading_conditions::LoadedCondition],
    marpol: bool,
) -> (
    Vec<ovcd_core::measured::DamageVerdict>,
    Vec<ovcd_core::measured::DamageVerdict>,
) {
    use ovcd_core::damage_stability::{DamageCriteria, assess};
    use ovcd_core::liquid_bulk::damage::{
        AdnExtents, DamageCase, damage_cases, marpol_damage_cases,
    };
    use ovcd_core::loading_conditions::Role;
    use ovcd_core::sections::Gravity;
    let adn = crate::trace::adn_vessel_type_of(cargo_key);
    if adn.is_none() && !marpol {
        return (Vec::new(), Vec::new());
    }
    let none = || (Vec::new(), Vec::new());
    let Some(laden) = conditions
        .iter()
        .find(|c| c.role == Role::FullLoadDeparture)
    else {
        return none();
    };
    // The mass and its centre of gravity: the weights of the condition. The damage does not change either.
    let mass: f64 = laden.weights.iter().map(|(_, i)| i.mass_t).sum();
    if mass.is_nan() || mass <= 0.0 {
        return none();
    }
    let g = Gravity {
        lcg_m: laden
            .weights
            .iter()
            .map(|(_, i)| i.mass_t * i.lcg_m)
            .sum::<f64>()
            / mass,
        tcg_m: 0.0,
        kg_m: laden
            .weights
            .iter()
            .map(|(_, i)| i.mass_t * i.kcg_m)
            .sum::<f64>()
            / mass,
    };
    let Ok(float) = hull.float(mass, g, density) else {
        return none();
    };
    let start = (
        float.equilibrium.waterplane.c,
        float.equilibrium.waterplane.trim_deg,
    );
    let verdicts = |cases: Vec<DamageCase>, criteria: &DamageCriteria| {
        cases
            .into_iter()
            .map(|case| ovcd_core::measured::DamageVerdict {
                flooded: case.flooded.iter().map(|f| f.name.clone()).collect(),
                outcome: assess(
                    &hull.table,
                    &case.flooded,
                    mass,
                    g,
                    density,
                    start,
                    criteria,
                ),
                name: case.name,
            })
            .collect::<Vec<_>>()
    };
    let by_adn = adn.map_or_else(Vec::new, |t| {
        let extents = if t == adn_vessel_type::AdnVesselType::C {
            AdnExtents::TYPE_C
        } else {
            AdnExtents::TYPE_N
        };
        verdicts(
            damage_cases(&liquid.spaces, hull.spec.loa, hull.spec.beam, &extents),
            &DamageCriteria::ADN_2025,
        )
    });
    let by_marpol = if marpol {
        verdicts(
            marpol_damage_cases(&liquid.spaces, hull.spec.loa, hull.spec.beam),
            &DamageCriteria::MARPOL_I_28,
        )
    } else {
        Vec::new()
    };
    (by_adn, by_marpol)
}

/// The `damage` block of the results: what each assumed damage does (unhashed, with the rest of the
/// Rule Trace inputs).
pub(crate) fn damage_json(verdicts: &[ovcd_core::measured::DamageVerdict]) -> Value {
    Value::Array(
        verdicts
            .iter()
            .map(|d| {
                let o = &d.outcome;
                json!({
                    "name": d.name,
                    "flooded": d.flooded,
                    "floats": match o.floats {
                        ovcd_core::damage_stability::Floats::Yes => "yes",
                        ovcd_core::damage_stability::Floats::Sinks => "sinks",
                        ovcd_core::damage_stability::Floats::NoEquilibrium => "no equilibrium",
                    },
                    "heel_deg": num(o.heel_deg),
                    "trim_deg": num(o.trim_deg),
                    "draught_mid_m": num(o.draught_mid_m),
                    "range_end_deg": num(o.end_deg),
                    "max_gz_m": num(o.max_gz_m),
                    "area_m_rad": num(o.area_m_rad),
                    "ok": o.ok,
                })
            })
            .collect(),
    )
}

/// The `oil_outflow` block of the results: OM of MARPOL Annex I regulation 23 and what it is made of (unhashed,
/// with the rest of the Rule Trace inputs).
pub(crate) fn oil_outflow_json(o: &ovcd_core::liquid_bulk::oil_outflow::OilOutflow) -> Value {
    json!({
        "cargo_volume_m3": num(o.c_m3),
        "nominal_density_kg_per_m3": num(o.rho_n),
        "mean_side_outflow_m3": num(o.oms_m3),
        "mean_bottom_outflow_tide_0_m3": num(o.omb_0_m3),
        "mean_bottom_outflow_tide_minus_2_5_m3": num(o.omb_25_m3),
        "mean_bottom_outflow_m3": num(o.omb_m3),
        "mean_outflow_parameter": num(o.om),
        "limit": num(o.limit),
        "ratio": num(o.ratio()),
        "tanks": o.tanks.iter().map(|t| json!({
            "name": t.name,
            "oil_m3": num(t.oil_m3),
            "side_probability_port": num(t.ps_port),
            "side_probability_starboard": num(t.ps_starboard),
            "bottom_probability_port": num(t.pb_port),
            "bottom_probability_starboard": num(t.pb_starboard),
            "bottom_outflow_tide_0_m3": num(t.ob_0_m3),
            "bottom_outflow_tide_minus_2_5_m3": num(t.ob_25_m3),
            "capture_factor": num(t.cdb),
        })).collect::<Vec<_>>(),
    })
}
