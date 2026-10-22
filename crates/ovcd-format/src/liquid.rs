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

/// Issue #341: the assumed damages of the hull and what each does to the laden departure condition, by the
/// ADN extents of the vessel type of the Cargo (9.3.2.15.1 for type C, 9.3.3.15.1 for type N). Empty when the
/// Cargo has no ADN type or the departure condition has no floating position.
pub(crate) fn damage_verdicts(
    hull: &ovcd_core::hydrostatics::Hull,
    liquid: &LiquidBulkResult,
    cargo_key: &str,
    density: f64,
    conditions: &[ovcd_core::loading_conditions::LoadedCondition],
) -> Vec<ovcd_core::measured::DamageVerdict> {
    use ovcd_core::damage_stability::{AdnCriteria, assess};
    use ovcd_core::liquid_bulk::damage::{AdnExtents, damage_cases};
    use ovcd_core::loading_conditions::Role;
    use ovcd_core::sections::Gravity;
    let Some(adn) = crate::trace::adn_vessel_type_of(cargo_key) else {
        return Vec::new();
    };
    let extents = if adn == adn_vessel_type::AdnVesselType::C {
        AdnExtents::TYPE_C
    } else {
        AdnExtents::TYPE_N
    };
    let Some(laden) = conditions
        .iter()
        .find(|c| c.role == Role::FullLoadDeparture)
    else {
        return Vec::new();
    };
    // The mass and its centre of gravity: the weights of the condition. The damage does not change either.
    let mass: f64 = laden.weights.iter().map(|(_, i)| i.mass_t).sum();
    if mass.is_nan() || mass <= 0.0 {
        return Vec::new();
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
        return Vec::new();
    };
    let start = (
        float.equilibrium.waterplane.c,
        float.equilibrium.waterplane.trim_deg,
    );
    damage_cases(&liquid.spaces, hull.spec.loa, hull.spec.beam, &extents)
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
                &AdnCriteria::ADN_2025,
            ),
            name: case.name,
        })
        .collect()
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
