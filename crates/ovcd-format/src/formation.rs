//! The powering of a pushed Formation with its pusher (issue #359, spec 0007 "Formations").
//!
//! The Option's inputs describe one barge, and `simulate` runs it as a pushed barge. When the Option
//! includes its tug (`tug_included`), this module adds the `formation` block to the result: the pusher
//! sized for the Formation, the Equivalent Hull, the power and the service speed in each environment,
//! the bollard pull and the coupling force. The result of the barge itself does not change.

use crate::simulate::formation_curves;
use crate::text_ids::TextIds;
use crate::vdesign::DesignOption;
use ovcd_core::formation::{
    EquivalentHull, MAX_PASSES, POWER_TOLERANCE, Sized, TugDims, coupling_force_kn,
    pushed_equivalent_hull, pusher_beam_m, pusher_draught_m, pusher_length_m, size_loop,
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
    })
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

/// The tug of a pass: its size for the power `power_kw`, its lightship, the Equivalent Hull with it,
/// and the powering that Formation needs.
fn pass(
    p: &Map<String, Value>,
    tug: &TugInputs,
    barge: (f64, f64, f64),
    barge_volume_m3: f64,
    counts: (u64, u64),
    rho: f64,
    power_kw: f64,
) -> Result<Pass, String> {
    let dims = TugDims {
        length_m: tug.length_m.unwrap_or_else(|| pusher_length_m(power_kw)),
        beam_m: tug.beam_m.unwrap_or_else(|| pusher_beam_m(power_kw)),
        draught_m: tug
            .draught_m
            .unwrap_or_else(|| pusher_draught_m(p.get("draught_limit_m").and_then(Value::as_f64))),
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
    let hull = pushed_equivalent_hull(barge, barge_volume_m3, counts, &dims, lightship_t / rho);
    // The resistance of the Equivalent Hull, times the allowance, at the speeds of the curves.
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
    let pushed = matches!(kind.as_deref(), Some("pushed_barge" | "pushed_convoy"));
    let included = option
        .input("tug_included")
        .is_some_and(|i| i.value == json!(true));
    if !(pushed && included) {
        return;
    }
    let mut ids = TextIds::default();
    let mut notes: Vec<ovcd_core::text::Text> = Vec::new();
    let block = match build(option, p, counts, v, &mut notes) {
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

fn build(
    option: &DesignOption,
    p: &Map<String, Value>,
    counts: (u64, u64),
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

    let mut last: Option<Pass> = None;
    let mut error: Option<String> = None;
    let sized: Sized = size_loop(tug.installed_power_kw.unwrap_or(START_KW), |kw| match pass(
        &p2,
        &tug,
        barge,
        barge_volume,
        counts,
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
    if !sized.converged {
        notes.push(text!(
            "warn-formation-sizing-not-converged",
            "The pusher sizing did not converge in {passes} passes (the power moved by more than {tol} %). The power shown is that of the last pass.",
            passes = Arg::Count(MAX_PASSES as i64),
            tol = Arg::fixed(POWER_TOLERANCE * 100.0, 0),
        ));
    }
    notes.push(text!(
        "note-formation-method-unverified",
        "The Formation's resistance is that of its Equivalent Hull (the barges and the pusher as one hull) by the single-hull method, times the Formation resistance allowance. The method is unverified."
    ));
    let force = coupling_force_kn(
        r.powering.installed_power_kw,
        r.dims.length_m,
        r.dims.beam_m,
    );
    notes.push(text!(
        "note-formation-coupling-force",
        "Coupling force between the pusher and the pushed craft: {force} kN (ES-TRIN 2025/1, ESI-III-6 2.1: 270 x P_B x L_S / B_S, with P_B {power} kW, L_S {ls} m taken as the pusher length, B_S {bs} m).",
        force = Arg::fixed(force, 0),
        power = Arg::fixed(r.powering.installed_power_kw, 0),
        ls = Arg::fixed(r.dims.length_m, 1),
        bs = Arg::fixed(r.dims.beam_m, 1),
    ));
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
    Ok(json!({
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
        "coupling_force_kn": crate::eval::num(force),
        "notes": notes.iter().map(|n| json!(n.en)).collect::<Vec<_>>(),
    }))
}
