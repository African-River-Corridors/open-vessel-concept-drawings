//! The powering of a pushed Formation with its pusher (issue #359) or an ATB with its tug (issue
//! #362), spec 0007 "Formations".
//!
//! The Option's inputs describe one barge, and `simulate` runs it as a pushed barge (an ATB barge
//! with its Notch). When the Option includes its tug (`tug_included`), this module adds the
//! `formation` block to the result: the tug sized for the Formation (a pusher by its power, an ATB
//! tug from the barge), the Equivalent Hull, the power and the service speed in each environment, the
//! bollard pull and, for a pusher, the coupling force. The result of the barge itself does not
//! change.

use crate::simulate::formation_curves;
use crate::text_ids::TextIds;
use crate::vdesign::DesignOption;
use ovcd_core::atb::{TugTyped, notch_for_tug, tug_from_barge};
use ovcd_core::formation::{
    ENGINE_MARGIN, EquivalentHull, MAX_PASSES, POWER_TOLERANCE, SEA_MARGIN, Sized, TugDims,
    atb_equivalent_hull, coupling_force_kn, pushed_equivalent_hull, pusher_beam_m,
    pusher_draught_m, pusher_length_m, size_loop, tow_resistance_kn, towing_tug_beam_m,
    towing_tug_draught_m, towing_tug_length_m,
};
use ovcd_core::machinery::{self, MachineryInputs, PowerSource};
use ovcd_core::powering::{self, PoweringInputs};
use ovcd_core::propulsor::{PropulsorInputs, PropulsorType, WAKE_OPEN_STERN, WAKE_TUNNEL};
use ovcd_core::stern_form::{K_D_OPEN, K_D_TUNNEL};
use ovcd_core::text;
use ovcd_core::text::Arg;
use serde_json::{Map, Value, json};

/// The first guess of the pusher's power, the reference of the size ratios (kW).
const START_KW: f64 = 1400.0;
/// The double bottom height of the pusher, for the centres of its machinery items (m, estimate).
const PUSHER_DOUBLE_BOTTOM_M: f64 = 0.5;
/// The drive room of a pusher with a drive that is not a shaft line (m), as the stern block's.
const PUSHER_DRIVE_ROOM_M: f64 = 3.5;

/// The tug inputs of an Option, as stored.
struct TugInputs {
    length_m: Option<f64>,
    beam_m: Option<f64>,
    draught_m: Option<f64>,
    installed_power_kw: Option<f64>,
    propulsor: PropulsorType,
    count: u32,
    diameter_m: Option<f64>,
    power_source: PowerSource,
    tunnel: bool,
    design_speed_kn: f64,
    margin: f64,
    allowance: f64,
    /// The tug's sightline is a Blind Distance over the convoy (`tug_sightline_method`).
    blind_distance: bool,
    /// The tug's wheelhouse lifts (`tug_wheelhouse_type`).
    elevating: bool,
}

fn number(option: &DesignOption, name: &str) -> Option<f64> {
    option.input(name).and_then(|i| i.value.as_f64())
}

fn tug_inputs(option: &DesignOption) -> Result<TugInputs, String> {
    let text_of = |name: &str| {
        option
            .input(name)
            .and_then(|i| i.value.as_str())
            .map(str::to_string)
    };
    let propulsor = text_of("tug_propulsor_type")
        .as_deref()
        .and_then(PropulsorType::from_key)
        .ok_or("the Option has no tug Propulsor type")?;
    let power_source = text_of("tug_power_source")
        .as_deref()
        .and_then(PowerSource::from_key)
        .ok_or("the Option has no tug Power Source")?;
    Ok(TugInputs {
        length_m: number(option, "tug_length_m"),
        beam_m: number(option, "tug_beam_m"),
        draught_m: number(option, "tug_draught_m"),
        installed_power_kw: number(option, "tug_installed_power_kw").filter(|p| *p > 0.0),
        propulsor,
        count: number(option, "tug_propulsor_count").map_or(2, |n| n.max(1.0) as u32),
        diameter_m: number(option, "tug_propeller_diameter_m"),
        power_source,
        tunnel: text_of("tug_stern_form").as_deref() == Some("tunnel"),
        design_speed_kn: number(option, "tug_design_speed_kn").unwrap_or(7.5),
        margin: number(option, "tug_installed_power_margin").unwrap_or(1.2),
        allowance: number(option, "formation_resistance_allowance").unwrap_or(1.0),
        blind_distance: text_of("tug_sightline_method").as_deref() == Some("blind_distance"),
        elevating: text_of("tug_wheelhouse_type").as_deref() != Some("fixed"),
    })
}

/// How the Formation is joined: the pusher at the stern of barges in line and abreast, or the ATB tug
/// in the Notch of one barge.
#[derive(Clone, Copy)]
enum Join {
    Pushed {
        counts: (u64, u64),
    },
    /// The tug's dimensions come from the barge, and the Equivalent Hull is shorter by the Notch.
    Atb {
        dims: TugDims,
        notch_depth_m: f64,
    },
    /// `barges` barges on a hawser behind the tug: separate hulls, resistances added (issue #361).
    Towed {
        barges: u64,
    },
}

/// What one pass of the sizing found.
#[derive(Clone)]
struct Pass {
    dims: TugDims,
    diameter_m: f64,
    hull: EquivalentHull,
    hull_steel_t: f64,
    machinery_t: f64,
    lightship_t: f64,
    powering: powering::Powering,
}

/// The curves of a tow: at each speed of the barge's curves, `barges` barges and the tug add, times the
/// allowance. The tug's curves are on the same speed grid (the same inputs make them), so a point that
/// does not line up is an error, not a guess.
fn add_tow_curves(
    mut barge: Vec<powering::EnvironmentCurve>,
    tug: &[powering::EnvironmentCurve],
    barges: u64,
    allowance: f64,
) -> Result<Vec<powering::EnvironmentCurve>, String> {
    for (b, t) in barge.iter_mut().zip(tug) {
        if b.points.len() != t.points.len() {
            return Err("the speed curves of the barge and the tug do not line up".to_string());
        }
        for (pb, pt) in b.points.iter_mut().zip(&t.points) {
            if (pb.speed_kn - pt.speed_kn).abs() > 1e-9 {
                return Err("the speed curves of the barge and the tug do not line up".to_string());
            }
            pb.resistance_kn =
                tow_resistance_kn(pb.resistance_kn, barges, pt.resistance_kn, allowance);
            pb.squat_limited |= pt.squat_limited;
        }
    }
    Ok(barge)
}

/// The tug of a pass: its size for the power `power_kw`, its lightship, the Equivalent Hull with it,
/// and the powering that Formation needs.
fn pass(
    p: &Map<String, Value>,
    tug: &TugInputs,
    barge: (f64, f64, f64),
    barge_volume_m3: f64,
    join: Join,
    rho: f64,
    power_kw: f64,
) -> Result<Pass, String> {
    let dims = match join {
        Join::Atb { dims, .. } => dims,
        Join::Pushed { .. } => TugDims {
            length_m: tug.length_m.unwrap_or_else(|| pusher_length_m(power_kw)),
            beam_m: tug.beam_m.unwrap_or_else(|| pusher_beam_m(power_kw)),
            draught_m: tug.draught_m.unwrap_or_else(|| {
                pusher_draught_m(p.get("draught_limit_m").and_then(Value::as_f64))
            }),
        },
        Join::Towed { .. } => {
            let length_m = tug
                .length_m
                .unwrap_or_else(|| towing_tug_length_m(power_kw));
            let beam_m = tug.beam_m.unwrap_or_else(|| towing_tug_beam_m(length_m));
            TugDims {
                length_m,
                beam_m,
                draught_m: tug
                    .draught_m
                    .unwrap_or_else(|| towing_tug_draught_m(beam_m)),
            }
        }
    };
    // The propeller: typed, else the largest the stern allows.
    let k_d = if tug.tunnel { K_D_TUNNEL } else { K_D_OPEN };
    let d_max = k_d * dims.draught_m;
    let diameter_m = tug.diameter_m.unwrap_or_else(|| {
        tug.propulsor
            .max_unit_diameter_m()
            .map_or(d_max, |m| d_max.min(m))
    });
    // The pusher's own lightship: hull steel by the box rate on its L x B x D, and its machinery.
    let hull_steel_t = ovcd_core::models::BargeInputs::new(1.0, 1.0, 1.0, 1.0, 1.0)
        .steel_rate_per_cbm
        * dims.length_m
        * dims.beam_m
        * dims.depth_m();
    let m_inputs = MachineryInputs {
        power_source: tug.power_source,
        propulsor: tug.propulsor,
        propulsor_count: tug.count,
        propeller_diameter_m: diameter_m,
        installed_power_kw: Some(power_kw),
        battery_energy_kwh: None,
        bow_thruster_kw: None,
        bow_thruster_lcg_m: dims.length_m,
        stern_run_m: 0.0,
    };
    let arranged = machinery::arrange(
        &m_inputs,
        dims.beam_m,
        dims.depth_m(),
        PUSHER_DRIVE_ROOM_M,
        None,
    );
    let machinery_t: f64 = machinery::mass_items(
        &m_inputs,
        &arranged,
        dims.depth_m(),
        PUSHER_DOUBLE_BOTTOM_M,
        power_kw,
    )
    .iter()
    .map(|i| i.mass_t)
    .sum();
    let lightship_t = hull_steel_t + machinery_t;
    let tug_volume = lightship_t / rho;
    let hull = match join {
        Join::Pushed { counts } => {
            pushed_equivalent_hull(barge, barge_volume_m3, counts, &dims, tug_volume)
        }
        Join::Atb { notch_depth_m, .. } => {
            atb_equivalent_hull(barge, barge_volume_m3, &dims, tug_volume, notch_depth_m)
        }
        // A tow has no Equivalent Hull: the barge stands for it, and the parts are run apart below.
        Join::Towed { .. } => EquivalentHull {
            length_m: barge.0,
            beam_m: barge.1,
            draught_m: barge.2,
            block_coefficient: barge_volume_m3 / (barge.0 * barge.1 * barge.2),
            displaced_volume_m3: barge_volume_m3,
        },
    };
    // The resistance of the Equivalent Hull, times the allowance, at the speeds of the curves.
    let curves = if let Join::Towed { barges } = join {
        // The barges and the tug are separate hulls: the resistance is the sum of the parts at each
        // speed, times the allowance.
        let barge_curves =
            formation_curves(p, barge, hull.block_coefficient, barge_volume_m3 * rho, rho)?
                .ok_or("the Formation has no draught")?;
        let tug_curves = formation_curves(
            p,
            (dims.length_m, dims.beam_m, dims.draught_m),
            tug_volume / (dims.length_m * dims.beam_m * dims.draught_m),
            tug_volume * rho,
            rho,
        )?
        .ok_or("the tug has no draught")?;
        add_tow_curves(barge_curves, &tug_curves, barges, tug.allowance)?
    } else {
        let mut curves = formation_curves(
            p,
            (hull.length_m, hull.beam_m, hull.draught_m),
            hull.block_coefficient,
            hull.displaced_volume_m3 * rho,
            rho,
        )?
        .ok_or("the Formation has no draught")?;
        for c in &mut curves {
            for pt in &mut c.points {
                pt.resistance_kn *= tug.allowance;
            }
        }
        curves
    };
    let inputs = PoweringInputs {
        propulsor: PropulsorInputs {
            propulsor_type: tug.propulsor,
            count: tug.count,
            diameter_m,
            wake: if tug.tunnel {
                WAKE_TUNNEL
            } else {
                WAKE_OPEN_STERN
            },
            kappa: p
                .get("propulsor_efficiency_factor")
                .and_then(Value::as_f64)
                .unwrap_or(1.0),
            kappa_b: p
                .get("bollard_pull_factor")
                .and_then(Value::as_f64)
                .unwrap_or(1.0),
            rho_kg_m3: rho * 1000.0,
        },
        power_source: tug.power_source,
        design_speed_kn: tug.design_speed_kn,
        margin: tug.margin,
        installed_power_kw: tug.installed_power_kw,
        trip_length_km: None,
    };
    Ok(Pass {
        dims,
        diameter_m,
        hull,
        hull_steel_t,
        machinery_t,
        lightship_t,
        powering: powering::compute(&inputs, &curves),
    })
}

/// Add the `formation` block of a pushed barge or convoy that includes its tug. Nothing is added
/// for any other Option, so a result with `tug_included` false is what it was.
pub(crate) fn add_formation(
    option: &DesignOption,
    p: &Map<String, Value>,
    counts: (u64, u64),
    v: &mut Value,
) {
    let kind = option
        .input("formation")
        .and_then(|i| i.value["kind"].as_str().map(str::to_string));
    let pushed = matches!(
        kind.as_deref(),
        Some("pushed_barge" | "pushed_convoy" | "atb" | "towed_barge")
    );
    let included = option
        .input("tug_included")
        .is_some_and(|i| i.value == json!(true));
    if !(pushed && included) {
        return;
    }
    let mut ids = TextIds::default();
    let mut notes: Vec<ovcd_core::text::Text> = Vec::new();
    let join_kind = match kind.as_deref() {
        Some("atb") => Kind::Atb,
        Some("towed_barge") => Kind::Towed,
        _ => Kind::Pushed,
    };
    let block = match build(option, p, counts, join_kind, v, &mut notes) {
        Ok(b) => b,
        Err(message) => json!({ "tug_included": true, "error": message }),
    };
    ids.texts(&notes);
    if let Some(o) = v.as_object_mut() {
        o.insert("formation".into(), block);
        if let Some(Value::Object(have)) = o.get_mut("text_ids")
            && let Value::Object(add) = ids.finish(&Value::Null)
        {
            for (k, val) in add {
                have.entry(k).or_insert(val);
            }
        }
    }
}

/// The warning of a sizing that stopped at the fifth pass without converging.
fn convergence_note(sized: &Sized) -> Option<ovcd_core::text::Text> {
    (!sized.converged).then(|| {
        text!(
            "warn-formation-sizing-not-converged",
            "The pusher sizing did not converge in {passes} passes (the power moved by more than {tol} %). The power shown is that of the last pass.",
            passes = Arg::Count(MAX_PASSES as i64),
            tol = Arg::fixed(POWER_TOLERANCE * 100.0, 0),
        )
    })
}

/// The kind of Formation the tug serves.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Pushed,
    Atb,
    Towed,
}

fn build(
    option: &DesignOption,
    p: &Map<String, Value>,
    counts: (u64, u64),
    kind: Kind,
    v: &mut Value,
    notes: &mut Vec<ovcd_core::text::Text>,
) -> Result<Value, String> {
    let tug = tug_inputs(option)?;
    let num = |path: &[&str]| -> Result<f64, String> {
        let mut cur = &*v;
        for k in path {
            cur = &cur[*k];
        }
        cur.as_f64()
            .ok_or_else(|| format!("the result has no {}", path.join(".")))
    };
    let barge = (
        num(&["dims", "loa_m"])?,
        num(&["dims", "beam_m"])?,
        num(&["results", "draught_m"])?,
    );
    let rho = p
        .get("water_density_t_per_m3")
        .and_then(Value::as_f64)
        .unwrap_or(1.0);
    let barge_volume = num(&["results", "displacement_t"])? / rho;
    // The speed range of the curves must hold the tug's Design Speed.
    let mut p2 = p.clone();
    let reach = p
        .get("speed_max_kn")
        .and_then(Value::as_f64)
        .unwrap_or(10.0)
        .max(tug.design_speed_kn + 1.0);
    p2.insert("speed_max_kn".into(), json!(reach));

    // An ATB tug comes from the barge (typed values kept) and its Notch from the tug; a pusher is
    // sized by its power in the loop.
    let atb = kind == Kind::Atb;
    let towed = kind == Kind::Towed;
    let join = if towed {
        // One barge on the hawser until the Formation holds a count (the Formation of a tow holds
        // only its kind).
        Join::Towed { barges: 1 }
    } else if atb {
        let dims = tug_from_barge(
            barge.0,
            barge.1,
            TugTyped {
                length_m: tug.length_m,
                beam_m: tug.beam_m,
                draught_m: tug.draught_m,
            },
        );
        let notch = notch_for_tug(
            &dims,
            number(option, "notch_depth_m"),
            number(option, "notch_width_m"),
        );
        Join::Atb {
            dims: TugDims {
                length_m: dims.length_m,
                beam_m: dims.beam_m,
                draught_m: dims.draught_m,
            },
            notch_depth_m: notch.depth_m,
        }
    } else {
        Join::Pushed { counts }
    };
    let mut last: Option<Pass> = None;
    let mut error: Option<String> = None;
    let sized: Sized = size_loop(tug.installed_power_kw.unwrap_or(START_KW), |kw| match pass(
        &p2,
        &tug,
        barge,
        barge_volume,
        join,
        rho,
        kw,
    ) {
        Ok(r) => {
            let next = r.powering.installed_power_kw;
            last = Some(r);
            next
        }
        Err(e) => {
            error = Some(e);
            kw
        }
    });
    if let Some(e) = error {
        return Err(e);
    }
    let r = last.ok_or("the sizing ran no pass")?;
    notes.extend(convergence_note(&sized));
    let mut notes2: Vec<ovcd_core::text::Text> = Vec::new();
    if towed {
        notes.push(text!(
            "note-tow-method-unverified",
            "The tow's resistance is the sum of the resistances of its barge and of its tug as separate hulls by the single-hull method, times the Formation resistance allowance. The method is unverified. The hawser and its interaction are not modelled."
        ));
    } else if atb {
        notes.push(text!(
            "note-atb-method-unverified",
            "The ATB's resistance is that of its Equivalent Hull (the barge and the tug as one hull, as long as the barge and the tug less the Notch the tug sits in) by the single-hull method, times the Formation resistance allowance. The method is unverified."
        ));
    } else {
        notes.push(text!(
            "note-formation-method-unverified",
            "The Formation's resistance is that of its Equivalent Hull (the barges and the pusher as one hull) by the single-hull method, times the Formation resistance allowance. The method is unverified."
        ));
    }
    let force = coupling_force_kn(
        r.powering.installed_power_kw,
        r.dims.length_m,
        r.dims.beam_m,
    );
    if !atb && !towed {
        notes.push(text!(
            "note-formation-coupling-force",
            "Coupling force between the pusher and the pushed craft: {force} kN (ES-TRIN 2025/1, ESI-III-6 2.1: 270 x P_B x L_S / B_S, with P_B {power} kW, L_S {ls} m taken as the pusher length, B_S {bs} m).",
            force = Arg::fixed(force, 0),
            power = Arg::fixed(r.powering.installed_power_kw, 0),
            ls = Arg::fixed(r.dims.length_m, 1),
            bs = Arg::fixed(r.dims.beam_m, 1),
        ));
    }
    // The service margins (issue #649): the still-water power in deep water at the design speed,
    // with the sea margin and the engine margin, beside the installed power. They do not size the
    // tug: its margin does.
    let still = r.powering.environments.first().map(|e| {
        (
            e.name.clone(),
            powering::needed_at(&e.points, tug.design_speed_kn),
        )
    });
    let service_margins = still.as_ref().map(|(name, p0)| {
        notes.push(text!(
            "note-formation-service-margins",
            "Service margins, shown and not used to size the tug: the still-water power in deep water at the design speed, {p0} kW, x the sea margin 1.15 x the engine margin 1/0.90 (MAN B&W, Basic Principles of Ship Propulsion, p. 27 and Fig. 17) = {total} kW, for an installed power of {installed} kW.",
            p0 = Arg::fixed(*p0, 0),
            total = Arg::fixed(p0 * SEA_MARGIN * ENGINE_MARGIN, 0),
            installed = Arg::fixed(r.powering.installed_power_kw, 0),
        ));
        json!({
            "environment": name,
            "still_water_power_kw": crate::eval::num(*p0),
            "sea_margin": crate::eval::num(SEA_MARGIN),
            "engine_margin": crate::eval::num(ENGINE_MARGIN),
            "power_with_margins_kw": crate::eval::num(p0 * SEA_MARGIN * ENGINE_MARGIN),
        })
    });
    let environments: Vec<Value> = r
        .powering
        .environments
        .iter()
        .map(|e| {
            json!({
                "name": e.name,
                "service_speed_kn": crate::eval::num(e.service_speed_kn),
                "power_at_service_kw": crate::eval::num(e.needed_at_service_kw),
            })
        })
        .collect();
    // The tug joins the totals of a convoy.
    if let Some(Value::Object(totals)) = v.get_mut("convoy").and_then(|c| c.get_mut("totals")) {
        let ls = totals.get("lightship_t").and_then(Value::as_f64);
        let disp = totals.get("displacement_t").and_then(Value::as_f64);
        let len = totals.get("overall_length_m").and_then(Value::as_f64);
        totals.insert("tug_lightship_t".into(), json!(r.lightship_t));
        totals.insert(
            "lightship_with_tug_t".into(),
            json!(ls.map(|x| x + r.lightship_t)),
        );
        totals.insert(
            "displacement_with_tug_t".into(),
            json!(disp.map(|x| x + r.lightship_t)),
        );
        totals.insert(
            "overall_length_with_tug_m".into(),
            json!(len.map(|x| x + r.dims.length_m)),
        );
    }
    if let Some(c) = v.get_mut("convoy").and_then(Value::as_object_mut) {
        c.insert("pusher_included".into(), json!(true));
    }
    let mut block = json!({
        "tug_included": true,
        "design_speed_kn": crate::eval::num(tug.design_speed_kn),
        "resistance_allowance": crate::eval::num(tug.allowance),
        "method_status": "unverified",
        "equivalent_hull": {
            "length_m": crate::eval::num(r.hull.length_m),
            "beam_m": crate::eval::num(r.hull.beam_m),
            "draught_m": crate::eval::num(r.hull.draught_m),
            "block_coefficient": crate::eval::num(r.hull.block_coefficient),
        },
        "tug": {
            "length_m": crate::eval::num(r.dims.length_m),
            "beam_m": crate::eval::num(r.dims.beam_m),
            "draught_m": crate::eval::num(r.dims.draught_m),
            "depth_m": crate::eval::num(r.dims.depth_m()),
            "propulsor_type": tug.propulsor.key(),
            "propulsor_count": tug.count,
            "propeller_diameter_m": crate::eval::num(r.diameter_m),
            "installed_power_kw": crate::eval::num(r.powering.installed_power_kw),
            "installed_power_sized": r.powering.sized,
            "bollard_pull_t": crate::eval::num(r.powering.bollard_pull_t),
            "hull_steel_t": crate::eval::num(r.hull_steel_t),
            "machinery_t": crate::eval::num(r.machinery_t),
            "lightship_t": crate::eval::num(r.lightship_t),
        },
        "sizing": {
            "passes": sized.passes,
            "converged": sized.converged,
            "max_passes": MAX_PASSES,
            "tolerance": crate::eval::num(POWER_TOLERANCE),
        },
        "environments": environments,
        "service_margins": service_margins,
        "coupling_force_kn": crate::eval::num(force),
        "notes": notes.iter().map(|n| json!(n.en)).collect::<Vec<_>>(),
    });
    // An ATB has no coupling force (its linkage is rigid and its loads are for the approval items) and
    // holds its Notch depth; the block of a pushed Formation keeps the keys it had.
    if let (Join::Atb { notch_depth_m, .. }, Some(o)) = (join, block.as_object_mut()) {
        o.remove("coupling_force_kn");
        o.insert("notch_depth_m".into(), crate::eval::num(notch_depth_m));
    }
    // A tow has no coupling force (a hawser, not a rigid link) and no Equivalent Hull: the parts are
    // the barge and the tug.
    if let (Join::Towed { barges }, Some(o)) = (join, block.as_object_mut()) {
        o.remove("coupling_force_kn");
        o.remove("equivalent_hull");
        o.insert("barges_towed".into(), json!(barges));
    }
    // Issue #360: the tug's view over the barges ahead, when its sightline is a Blind Distance.
    if tug.blind_distance
        && let Some(s) = sightline(&tug, &r.dims, join, barge.0, v, &mut notes2)?
        && let Some(o) = block.as_object_mut()
    {
        o.insert("sightline".into(), s);
        if let Some(Value::Array(list)) = o.get_mut("notes") {
            list.extend(notes2.iter().map(|n| json!(n.en)));
        }
    }
    notes.extend(notes2);
    Ok(block)
}

/// The sightline of a pusher or an ATB tug over its barges (issue #360, spec 0007 "Wheelhouse,
/// sightline and air draft"): the eye at the forward face of the wheelhouse, at mid-length of the tug
/// (an Estimate), the floor lowered on the tug's deck; each barge's highest point light and laden
/// (its air draft) at its forward end; the ES-TRIN 7.02(2) limit, light. `None` for a tow (the tug
/// is ahead of its barge). With `notes` the warning when the lift is not enough.
fn sightline(
    tug: &TugInputs,
    dims: &TugDims,
    join: Join,
    barge_length_m: f64,
    v: &Value,
    notes: &mut Vec<ovcd_core::text::Text>,
) -> Result<Option<Value>, String> {
    use ovcd_core::aft_arrangement::{
        DEFAULT_WHEELHOUSE_EYE_ABOVE_FLOOR_M, DEFAULT_WHEELHOUSE_MAX_LIFT_M,
    };
    use ovcd_core::formation::{
        ConvoySight, PUSHER_WHEELHOUSE_AT, convoy_obstructions, convoy_sightline,
    };
    use ovcd_core::wheelhouse::ES_TRIN_BLIND_DISTANCE_M;
    let (first_aft_x, in_line) = match join {
        Join::Pushed { counts } => (dims.length_m, counts.1),
        Join::Atb { notch_depth_m, .. } => (dims.length_m - notch_depth_m, 1),
        Join::Towed { .. } => return Ok(None),
    };
    let air = |key: &str| {
        v["results"][key]
            .as_f64()
            .ok_or_else(|| format!("the result has no results.{key}"))
    };
    let light = convoy_obstructions(
        first_aft_x,
        barge_length_m,
        in_line,
        air("air_draft_light_m")?,
    );
    let laden = convoy_obstructions(
        first_aft_x,
        barge_length_m,
        in_line,
        air("air_draft_loaded_m")?,
    );
    let eye_height = DEFAULT_WHEELHOUSE_EYE_ABOVE_FLOOR_M;
    let max_lift = if tug.elevating {
        DEFAULT_WHEELHOUSE_MAX_LIFT_M
    } else {
        0.0
    };
    let sight = ConvoySight {
        eye_x_m: PUSHER_WHEELHOUSE_AT * dims.length_m,
        eye_lowered_m: dims.depth_m() - dims.draught_m + eye_height,
        eye_height_m: eye_height,
        max_lift_m: max_lift,
        bow_x_m: first_aft_x + in_line as f64 * barge_length_m,
    };
    let limit = ES_TRIN_BLIND_DISTANCE_M;
    let (s, laden_m) = convoy_sightline(&sight, &light, &laden, limit);
    if !s.enough {
        notes.push(text!(
            "warn-formation-blind-distance-lift",
            "The tug's view over the barges needs {needed} m of wheelhouse lift; the tug gives {max_lift} m. The Blind Distance light is {blind} m, more than {limit} m (ES-TRIN 7.02(2)).",
            needed = Arg::fixed(s.lift_needed_m, 2),
            max_lift = Arg::fixed(max_lift, 2),
            blind = Arg::fixed(s.blind_distance_m, 1),
            limit = Arg::fixed(limit, 0),
        ));
    }
    let num = crate::eval::num;
    Ok(Some(json!({
        "limit_m": num(limit),
        "eye_x_m": num(sight.eye_x_m),
        "eye_lowered_above_water_m": num(sight.eye_lowered_m),
        "eye_raised_above_water_m": num(s.eye_raised_m),
        "floor_raised_above_water_m": num(s.floor_raised_m),
        "lift_needed_m": num(s.lift_needed_m),
        "lift_used_m": num(s.lift_used_m),
        "max_lift_m": num(max_lift),
        "bow_x_m": num(sight.bow_x_m),
        "blind_distance_light_m": num(s.blind_distance_m),
        "blind_distance_laden_m": num(laden_m),
        "enough": s.enough,
    })))
}

/// The Notch and the tug of an ATB as the drawings need them (`drawing.atb`, issue #363): from the
/// same inputs and ratios the hull and the powering use. Nothing for another Formation. `drawing` is
/// not in the results hash.
pub(crate) fn add_atb_drawing(option: &DesignOption, v: &mut Value) {
    let atb = option
        .input("formation")
        .is_some_and(|i| i.value["kind"] == "atb");
    if !atb {
        return;
    }
    let (Some(l), Some(b)) = (number(option, "length_overall_m"), number(option, "beam_m")) else {
        return;
    };
    let dims = ovcd_core::atb::tug_from_barge(
        l,
        b,
        ovcd_core::atb::TugTyped {
            length_m: number(option, "tug_length_m"),
            beam_m: number(option, "tug_beam_m"),
            draught_m: number(option, "tug_draught_m"),
        },
    );
    let notch = ovcd_core::atb::notch_for_tug(
        &dims,
        number(option, "notch_depth_m"),
        number(option, "notch_width_m"),
    );
    let included = option
        .input("tug_included")
        .is_some_and(|i| i.value == json!(true));
    let tug = included.then(|| {
        let d = TugDims {
            length_m: dims.length_m,
            beam_m: dims.beam_m,
            draught_m: dims.draught_m,
        };
        json!({
            "length_m": d.length_m,
            "beam_m": d.beam_m,
            "draught_m": d.draught_m,
            "depth_m": d.depth_m(),
        })
    });
    if let Some(Value::Object(drawing)) = v.get_mut("drawing") {
        drawing.insert(
            "atb".into(),
            json!({
                "notch_depth_m": notch.depth_m,
                "notch_width_m": notch.width_m,
                "tug": tug,
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curves_of_a_tow_add_the_parts_at_every_speed() {
        use ovcd_core::powering::{CurvePoint, EnvironmentCurve};
        let curve = |name: &str, r: &[f64]| EnvironmentCurve {
            name: name.to_string(),
            points: r
                .iter()
                .enumerate()
                .map(|(i, r)| CurvePoint {
                    speed_kn: 1.0 + i as f64,
                    speed_through_water_kn: 1.0 + i as f64,
                    resistance_kn: *r,
                    squat_limited: false,
                })
                .collect(),
        };
        let barge = vec![curve("Deep water", &[10.0, 20.0, 40.0])];
        let tug = vec![curve("Deep water", &[1.0, 3.0, 9.0])];
        let sum = add_tow_curves(barge.clone(), &tug, 2, 1.0).unwrap();
        let r: Vec<f64> = sum[0].points.iter().map(|p| p.resistance_kn).collect();
        assert_eq!(r, [21.0, 43.0, 89.0]);
        // The allowance multiplies the sum.
        let more = add_tow_curves(barge.clone(), &tug, 1, 1.5).unwrap();
        assert!((more[0].points[2].resistance_kn - 73.5).abs() < 1e-12);
        // Curves on different grids are an error, not a guess.
        let short = vec![curve("Deep water", &[1.0, 3.0])];
        assert!(add_tow_curves(barge, &short, 1, 1.0).is_err());
    }

    #[test]
    fn a_sizing_that_stopped_at_five_passes_gives_a_warning_with_a_message_id() {
        let stopped = Sized {
            power_kw: 1000.0,
            passes: MAX_PASSES,
            converged: false,
        };
        let note = convergence_note(&stopped).expect("a warning");
        assert_eq!(
            note.msg.as_ref().unwrap().id,
            "warn-formation-sizing-not-converged"
        );
        assert!(
            note.en.contains("did not converge in 5 passes"),
            "{}",
            note.en
        );
        // A sizing that converged gives none.
        let done = Sized {
            converged: true,
            passes: 3,
            ..stopped
        };
        assert!(convergence_note(&done).is_none());
    }
}
