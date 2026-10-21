//! `simulate` and `cargo_options`: the one-call barge simulation, a port of `simulate.py`.
//!
//! It lives in `ovcd-format`, not `ovcd-core`: it takes and returns plain JSON (`params` in, a
//! result object out), the single entry point the future app calls. The Python `simulate` reads its
//! `params` mapping loosely (`float(p["loa_m"])` takes an int or numeric text, `p.get(..) or default`
//! takes any falsy value as absent). This port reads numbers, booleans and numeric text the same way
//! (text that is not a number is a recorded `ValueError`), truthiness the same way, and refuses (as `EvalError::BadInput`) the inputs where Python would raise `KeyError` or
//! `TypeError`, which the old Python callers never recorded either.
//!
//! A result that Python returns as `{"error": "..."}` (a cargo in the wrong vessel, a capacity
//! `ValueError` in the dry-bulk path) is an ordinary result here too. An exception that escapes
//! `simulate` (every `ValueError` from the gas path) is a recorded error.

use crate::EvalError;
use crate::eval::{aft_json, checks_json, gas_separation_json, num, ok, raised};
use crate::gas_stability::condition_json;
use crate::text_ids::TextIds;
use ovcd_core::barge_capacity::{self, BargeCapacityInputs, BargeCapacityResult};
use ovcd_core::cargo::{CARGO_DATABASE, CargoClass, CargoProperties};
use ovcd_core::checks::Check;
use ovcd_core::gas_arrangement::{self as ga, DEFAULT_DEPTHS_M, FitOptions, RULESETS};
use ovcd_core::gas_separation::GasSeparationInputs;
use ovcd_core::gas_stability::{self as gs, GasCapacityResult};
use ovcd_core::pyfmt::py_str;
use ovcd_core::pystr::{parse_float, repr_str};
use ovcd_core::resistance::{self, EnvironmentInputs, ResistanceInputs, TugInputs};
use ovcd_core::text;
use ovcd_core::text::{Arg, Text};
use serde_json::{Map, Value, json};

const MINUS: &str = "\u{2212}";

/// Why a simulation stopped before it had a result.
enum Stop {
    /// A refusal: the input is outside what this port models.
    Eval(EvalError),
    /// A Python exception that escapes `simulate`.
    Raised(String, String),
}

impl From<EvalError> for Stop {
    fn from(e: EvalError) -> Self {
        Stop::Eval(e)
    }
}

type R<T> = Result<T, Stop>;

fn bad<T>(msg: impl Into<String>) -> R<T> {
    Err(Stop::Eval(EvalError::BadInput(msg.into())))
}

/// Python truthiness of a JSON value.
fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// Python `float(v)` on a number, a boolean or text. Text that is not a number raises
/// `ValueError`, as in Python. Null and containers are refused (Python raises `TypeError`).
fn py_float(v: &Value, what: &str) -> R<f64> {
    match v {
        Value::Number(n) => Ok(n.as_f64().unwrap_or(f64::NAN)),
        Value::Bool(b) => Ok(f64::from(u8::from(*b))),
        Value::String(s) => parse_float(s).ok_or_else(|| {
            Stop::Raised(
                "ValueError".to_string(),
                format!("could not convert string to float: {}", repr_str(s)),
            )
        }),
        _ => bad(format!("`{what}` must be a number")),
    }
}

/// Python `repr(v)` of a JSON value, for the text of an error message.
fn py_repr(v: &Value) -> String {
    match v {
        Value::Null => "None".to_string(),
        Value::Bool(true) => "True".to_string(),
        Value::Bool(false) => "False".to_string(),
        Value::Number(n) if n.is_f64() => py_str(n.as_f64().unwrap_or(f64::NAN)),
        Value::Number(n) => n.to_string(),
        Value::String(s) => repr_str(s),
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(py_repr).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(map) => format!(
            "{{{}}}",
            map.iter()
                .map(|(k, x)| format!("{}: {}", repr_str(k), py_repr(x)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// `float(p[key])`.
fn need(p: &Map<String, Value>, key: &str) -> R<f64> {
    match p.get(key) {
        Some(v) => py_float(v, &format!("params.{key}")),
        None => bad(format!("`params.{key}` is required")),
    }
}

/// `float(p.get(key, default))`: only an absent key takes the default; null is refused.
fn get_or(p: &Map<String, Value>, key: &str, default: f64) -> R<f64> {
    match p.get(key) {
        None => Ok(default),
        Some(v) => py_float(v, &format!("params.{key}")),
    }
}

/// A count read as a number: a whole number within `i64`, or refused.
fn whole_number(x: f64, what: &str) -> R<i64> {
    if x.fract() != 0.0 || !x.is_finite() || x.abs() > 1e15 {
        return bad(format!("`params.{what}` must be a whole number"));
    }
    Ok(x as i64)
}

/// `p.get(key)` as the truthy-checked number: `Some(float(..))` when truthy, else `None`.
fn truthy_float(p: &Map<String, Value>, key: &str) -> R<Option<f64>> {
    match p.get(key) {
        Some(v) if truthy(v) => Ok(Some(py_float(v, &format!("params.{key}"))?)),
        _ => Ok(None),
    }
}

/// Python `min(items, key=value)`: the first of equal values (NaN never replaces the first).
fn min_item(items: &[(&str, f64)]) -> (String, f64) {
    let mut best = items[0];
    for &(n, v) in &items[1..] {
        if v < best.1 {
            best = (n, v);
        }
    }
    (best.0.to_string(), best.1)
}

/// Python float `a / b`: a zero divisor raises `ZeroDivisionError`.
fn py_div(a: f64, b: f64) -> R<f64> {
    if b == 0.0 {
        return Err(Stop::Raised(
            "ZeroDivisionError".to_string(),
            "float division by zero".to_string(),
        ));
    }
    Ok(a / b)
}

// Mutation survivors (#64), `>` to `>=` in `py_max` and `<` to `<=` in `py_min`: equivalent. They
// differ only on a tie, and on a tie both arguments are the same number, so either is returned.
// (A tie of 0.0 and -0.0 would differ in sign only; neither caller can make a -0.0.)
fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

fn py_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

fn cargo_entry(key: &str) -> R<&'static CargoProperties> {
    CARGO_DATABASE
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, c)| c)
        .map_or_else(|| bad(format!("unknown cargo `{key}`")), Ok)
}

/// `cargo_options()`.
fn cargo_options() -> Value {
    Value::Array(
        CARGO_DATABASE
            .iter()
            .map(|(k, c)| {
                json!({
                    "id": k,
                    "name": c.name,
                    "phase": c.phase.key(),
                    "density_kg_m3": num(c.density),
                })
            })
            .collect(),
    )
}

/// The `VESSELS` table: (label, hold type for the core, allowed Cargo Class).
fn vessel(id: &str) -> R<(&'static str, Option<&'static str>, CargoClass)> {
    Ok(match id {
        "box_barge" => ("Box-hold dry bulk barge", Some("box"), CargoClass::DryBulk),
        "hopper_barge" => (
            "Sloped-hopper dry bulk barge",
            Some("sloped_hopper"),
            CargoClass::DryBulk,
        ),
        "gas_barge" => ("Gas barge, IMO Type C tanks", None, CargoClass::Gas),
        other => return bad(format!("unknown vessel `{other}`")),
    })
}

fn error_result(message: String) -> Value {
    json!({ "error": message })
}

/// `_env`.
fn env(
    name: &str,
    depth: f64,
    bottom_w: f64,
    slope: f64,
    current_kn: f64,
    deep: bool,
    (rho, squat_margin_m): (f64, f64),
) -> EnvironmentInputs {
    let surface = bottom_w + 2.0 * slope * depth;
    // Mutation survivors (#64), in the struct below: deleting `bank_slope` and deleting
    // `apply_squat_limit` are equivalent. The core never reads `bank_slope` (the surface width,
    // set above, carries the slope), and `apply_squat_limit` defaults to `true`, which is what the
    // channel gets; deep water gets `false`, and the core does not apply squat in deep water
    // whatever this flag says (`squat_applies` also tests `is_deep_water`).
    EnvironmentInputs {
        name: name.to_string(),
        water_depth: depth,
        channel_width_bottom: bottom_w,
        channel_width_surface: surface,
        bank_slope: slope,
        current_speed_knots: current_kn,
        water_density: rho,
        squat_safety_margin_m: squat_margin_m,
        is_deep_water: deep,
        apply_squat_limit: !deep,
        ..EnvironmentInputs::default()
    }
}

fn channel(p: &Map<String, Value>) -> R<Map<String, Value>> {
    match p.get("channel") {
        Some(v) if truthy(v) => match v {
            Value::Object(o) => Ok(o.clone()),
            _ => bad("`params.channel` must be an object"),
        },
        _ => Ok(Map::new()),
    }
}

/// The channel entry of the limits table: `depth_m - ukc_m`.
fn channel_limit(ch: &Map<String, Value>) -> R<Option<f64>> {
    match ch.get("depth_m") {
        Some(v) if truthy(v) => {
            let depth = py_float(v, "params.channel.depth_m")?;
            let ukc = get_or(ch, "ukc_m", 0.5)?;
            Ok(Some(depth - ukc))
        }
        _ => Ok(None),
    }
}

fn h_over_t(ch: &Map<String, Value>, t: f64) -> R<Value> {
    match ch.get("depth_m") {
        // Mutation survivor (#64), `t > 0.0` to `t >= 0.0`: equivalent. `t` is the loaded draught,
        // which is above zero for every hull: displacement over `cb * l * b` for a barge, and the
        // light draught the hydrostatics gives (never zero, even for a 0.5 m hull) for a hull
        // that carries nothing.
        Some(v) if truthy(v) && t > 0.0 => Ok(num(py_float(v, "params.channel.depth_m")? / t)),
        _ => Ok(Value::Null),
    }
}

/// `_curves`: speed-power for the hull alone.
fn curves(
    p: &Map<String, Value>,
    l: f64,
    b: f64,
    t: f64,
    (cb, extra_wetted_m2): (f64, f64),
    disp: f64,
    rho: f64,
) -> R<Value> {
    let margin = get_or(
        p,
        "squat_safety_margin_m",
        EnvironmentInputs::default().squat_safety_margin_m,
    )?;
    let ch = channel(p)?;
    let mut envs = vec![env(
        "Deep water",
        50.0,
        1000.0,
        3.0,
        0.0,
        true,
        (rho, margin),
    )];
    if let (Some(d), Some(w)) = (ch.get("depth_m"), ch.get("bottom_width_m"))
        && truthy(d)
        && truthy(w)
    {
        envs.insert(
            0,
            env(
                "Channel",
                py_float(d, "params.channel.depth_m")?,
                py_float(w, "params.channel.bottom_width_m")?,
                get_or(&ch, "bank_slope", 3.0)?,
                get_or(&ch, "current_kn", 0.0)?,
                false,
                (rho, margin),
            ),
        );
    }
    if t <= 0.0 {
        return Ok(json!([]));
    }
    // Mutation survivors (#64), deleting one of these fields below: equivalent, each for its own
    // reason. `loa` and `displacement_tonnes` are never read by the resistance code (it uses `lwl`,
    // set to the same length, and works out the displacement itself). `speed_step_knots` is 0.5,
    // which is also the default. `engine_power_kw` (1e6 here, 2400 by default) only sets whether a
    // speed is "power available", which feeds the service speed; `simulate` lists neither, and the
    // curve is not cut by it.
    let inputs = ResistanceInputs {
        loa: l,
        lwl: l,
        beam: b,
        draft: t,
        block_coefficient: cb,
        displacement_tonnes: disp,
        tug: TugInputs {
            engine_power_kw: 1e6,
            propulsive_efficiency: get_or(p, "eta_d", 0.45)?,
            ..TugInputs::default()
        },
        environments: envs,
        speed_min_knots: 1.0,
        speed_max_knots: get_or(p, "speed_max_kn", 10.0)?,
        speed_step_knots: 0.5,
        // Issue #350: the wetted surface the Stern Form adds (a tunnel's roof, a run's slope).
        wetted_surface_extra_m2: extra_wetted_m2,
        ..ResistanceInputs::default()
    };
    let rr = resistance::compute(&inputs)
        .map_err(|e| Stop::Raised(e.type_name().to_string(), e.message().to_string()))?;
    Ok(Value::Array(
        rr.environments
            .iter()
            .map(|e| {
                json!({
                    "name": e.name,
                    "limiting_speed_kn": e.limiting_speed_knots.map_or(Value::Null, num),
                    "blockage": num(e.blockage_ratio),
                    "points": e.speed_curve.iter().map(|s| json!({
                        "kn": num(s.speed_knots),
                        "kw": num(s.delivered_power_kw),
                        "fuel_lph": num(s.fuel_litres_per_hour),
                        "squat_m": num(s.squat_m),
                        "squat_limited": s.squat_limited,
                    })).collect::<Vec<_>>(),
                })
            })
            .collect(),
    ))
}

/// The capacity checks as the simulator lists them: two superseded ones are left out, and the
/// name reads with spaces.
fn capacity_checks(r: &BargeCapacityResult, out: &mut Vec<Value>) {
    let kept: Vec<Check> = r
        .checks
        .iter()
        .filter(|c| c.name != "not_overloaded" && c.name != "freeboard_ok")
        .map(|c| Check {
            name: c.name.replace('_', " "),
            ..c.clone()
        })
        .collect();
    if let Value::Array(items) = checks_json(&kept) {
        out.extend(items);
    }
}

fn air_draft_check(r: &BargeCapacityResult, limit: f64, ids: &mut TextIds) -> R<Value> {
    let Some(light) = r.air_draft_light_m else {
        return bad("the capacity result has no light air draft");
    };
    let message = text!(
        "check-air-draft-limit",
        "{air_draft} m vs limit {limit} m",
        air_draft = Arg::fixed(light, 2),
        limit = Arg::fixed(limit, 2),
    );
    ids.text(&message);
    Ok(json!({
        "name": "Air draft (empty, governing)",
        "ok": light <= limit,
        "message": message,
    }))
}

fn opt_int(x: Option<i64>) -> Value {
    x.map_or(Value::Null, |i| json!(i))
}

fn opt_num(x: Option<f64>) -> Value {
    x.map_or(Value::Null, num)
}

/// `simulate(params)`.
/// The structural values the core already computes, for the GA view (#192). Nothing is worked out
/// here: each key is read from the core's result or inputs, and is `null` where the core has no
/// such value on this path.
///
/// - Gas: the double bottom, support, dome and wing tank values come from the converged
///   arrangement (`gas_basis`); `null` for a hull too small to hold one.
/// - `separation` is the core's gas separation result (gas only). `opening_setback_m` and
///   `fwd_end_bulkhead_m` are the separation inputs it was computed from.
/// - `aft_arrangement` is the core's aft block: `null` for a pushed barge, which has none.
/// - Dry bulk: `double_bottom_m` is the inner bottom height, `hatch_coaming_height_m` the coaming.
fn structure_json(
    r: &BargeCapacityResult,
    separation_inputs: Option<&GasSeparationInputs>,
    dry: Option<&BargeCapacityInputs>,
) -> Vec<(&'static str, Value)> {
    let basis = r.gas_basis.as_ref().map(|b| &b.inputs);
    let sep_in = separation_inputs.cloned().unwrap_or_default();
    let is_gas = r.separation.is_some();
    vec![
        (
            "double_bottom_m",
            match (basis, dry) {
                (Some(b), _) => opt_num(b.double_bottom_height),
                (None, Some(i)) => num(i.double_bottom_height_bulk),
                _ => Value::Null,
            },
        ),
        (
            "hatch_coaming_height_m",
            dry.map_or(Value::Null, |i| num(i.coaming_height)),
        ),
        (
            "tank_support_height_m",
            basis.map_or(Value::Null, |b| num(b.tank_support_height)),
        ),
        (
            "dome_height_m",
            basis.map_or(Value::Null, |b| num(b.dome_height)),
        ),
        (
            "wing_tank_width_m",
            basis.map_or(Value::Null, |b| opt_num(b.wing_tank_width)),
        ),
        (
            "wing_tank_gap_m",
            basis.map_or(Value::Null, |b| num(b.wing_tank_gap)),
        ),
        (
            "wing_tank_height_ratio",
            basis.map_or(Value::Null, |b| num(b.wing_tank_height_ratio)),
        ),
        (
            "opening_setback_m",
            if is_gas {
                num(sep_in.opening_setback_m)
            } else {
                Value::Null
            },
        ),
        (
            "fwd_end_bulkhead_m",
            if is_gas {
                num(sep_in.fwd_end_bulkhead_m)
            } else {
                Value::Null
            },
        ),
        (
            "separation",
            r.separation
                .as_ref()
                .map_or(Value::Null, gas_separation_json),
        ),
        (
            "aft_arrangement",
            r.aft_arrangement.as_ref().map_or(Value::Null, aft_json),
        ),
    ]
}

/// The `drawing` object: the structural values as an object, kept out of `geometry` and out of the
/// results hash.
fn drawing_json(extra: Vec<(&'static str, Value)>) -> Value {
    Value::Object(extra.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

/// The stern block's design values from the params. A key left out keeps the core default, the
/// value the engine used before these were inputs (the unit counts keep the Cargo phase default).
fn aft_design_of(p: &Map<String, Value>) -> R<ovcd_core::aft_arrangement::AftDesign> {
    use ovcd_core::aft_arrangement as a;
    let d = a::AftDesign::default();
    let whole =
        |key: &str, default: i64| -> R<i64> { whole_number(get_or(p, key, default as f64)?, key) };
    let units = |key: &str| -> R<Option<i64>> {
        match p.get(key) {
            None => Ok(None),
            Some(_) => Ok(Some(whole(key, 0)?)),
        }
    };
    Ok(a::AftDesign {
        n_thrusters: whole("thruster_count", d.n_thrusters)?,
        thruster_envelope_m: get_or(p, "thruster_envelope_m", d.thruster_envelope_m)?,
        thruster_clear_m: get_or(p, "thruster_clearance_m", d.thruster_clear_m)?,
        thruster_room_length_m: get_or(p, "thruster_room_length_m", d.thruster_room_length_m)?,
        n_battery_units: units("battery_unit_count")?,
        n_accom_units: units("accommodation_unit_count")?,
        container_height_m: get_or(p, "unit_height_m", d.container_height_m)?,
        container_side_gap_m: get_or(p, "unit_side_gap_m", d.container_side_gap_m)?,
        container_gap_m: get_or(p, "unit_row_gap_m", d.container_gap_m)?,
        container_foundation_m: get_or(p, "unit_foundation_m", d.container_foundation_m)?,
        wheelhouse_length_m: get_or(p, "wheelhouse_length_m", d.wheelhouse_length_m)?,
        wheelhouse_width_m: get_or(p, "wheelhouse_width_m", d.wheelhouse_width_m)?,
        wheelhouse_height_m: get_or(p, "wheelhouse_height_m", d.wheelhouse_height_m)?,
        wheelhouse_gap_m: get_or(p, "wheelhouse_gap_m", d.wheelhouse_gap_m)?,
        wheelhouse_eye_above_floor_m: get_or(
            p,
            "wheelhouse_eye_height_m",
            d.wheelhouse_eye_above_floor_m,
        )?,
        sightline_margin_m: get_or(p, "sightline_margin_m", d.sightline_margin_m)?,
        wheelhouse_max_lift_m: get_or(p, "wheelhouse_max_lift_m", d.wheelhouse_max_lift_m)?,
    })
}

/// The tank wall thickness of the params (issue #292). A key left out keeps the fixed 25 mm the
/// engine used before the input was optional; `null` asks for the pressure-vessel formula, which is
/// what a Design takes from the Defaults Profile.
fn tank_wall_of(p: &Map<String, Value>) -> R<Option<f64>> {
    match p.get("tank_wall_thickness_mm") {
        None => Ok(Some(
            ovcd_core::barge_capacity::DEFAULT_TANK_WALL_THICKNESS_MM,
        )),
        Some(Value::Null) => Ok(None),
        Some(v) => Ok(Some(py_float(v, "params.tank_wall_thickness_mm")?)),
    }
}

/// A dome size of the params (issue #316). A key left out keeps the fixed 2.0 m the engine used
/// before; `null` (`None` here) asks for the dome rule, which is what a Design takes from the
/// Defaults Profile.
fn dome_of(p: &Map<String, Value>, key: &str) -> R<Option<f64>> {
    match p.get(key) {
        None => Ok(Some(match key {
            "tank_dome_height_m" => ovcd_core::barge_capacity::DEFAULT_TANK_DOME_HEIGHT_M,
            _ => ovcd_core::barge_capacity::DEFAULT_TANK_DOME_DIAMETER_M,
        })),
        Some(Value::Null) => Ok(None),
        Some(v) => Ok(Some(py_float(v, &format!("params.{key}"))?)),
    }
}

/// The Cargo-dependent tank design values from the params (issue #292). A key left out keeps what
/// the engine used before these were inputs; `tank_allowable_stress_mpa` null or left out means the
/// material's own.
fn tank_design_of(p: &Map<String, Value>) -> R<ovcd_core::barge_capacity::TankDesign> {
    use ovcd_core::barge_capacity as b;
    let material = match p.get("tank_material") {
        None => b::DEFAULT_TANK_MATERIAL.to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => {
            return bad(format!(
                "`params.tank_material` must be text, not {}",
                py_repr(other)
            ));
        }
    };
    let allowable = match p.get("tank_allowable_stress_mpa") {
        None | Some(Value::Null) => None,
        Some(v) => Some(py_float(v, "params.tank_allowable_stress_mpa")?),
    };
    // Issue #313: with no typed stress, the basis picks the material's IGC 4.23.3.1 stress or the
    // stress of a file saved before that check (left out: the old one).
    let allowable = match (allowable, p.get("tank_allowable_stress_basis")) {
        (Some(f), _) => Some(f),
        (None, Some(Value::String(s))) if s == "igc_4_23_3_1" => None,
        (None, None) => published_design_stress(&material),
        (None, Some(Value::String(s))) if s == "published_design" => {
            published_design_stress(&material)
        }
        (None, Some(other)) => {
            return bad(format!(
                "`params.tank_allowable_stress_basis` must be one of igc_4_23_3_1, \
                 published_design, not {}",
                py_repr(other)
            ));
        }
    };
    // Issue #374: the design liquid pressure; left out, the vapour pressure only (the old way).
    let liquid_pressure = match p.get("tank_liquid_pressure") {
        None | Some(Value::Null) => b::LiquidPressure::None,
        Some(Value::String(s)) if s == "none" => b::LiquidPressure::None,
        Some(Value::String(s)) if s == "static" => b::LiquidPressure::Static,
        Some(Value::String(s)) if s == "igc_4_28_2" => b::LiquidPressure::Igc4282 {
            service_speed_kn: get_or(p, "max_speed_kn", 10.0)?,
        },
        Some(other) => {
            return bad(format!(
                "`params.tank_liquid_pressure` must be one of static, igc_4_28_2, none, not {}",
                py_repr(other)
            ));
        }
    };
    let design_temperature_c = match p.get("tank_design_temperature_c") {
        None | Some(Value::Null) => None,
        Some(v) => Some(py_float(v, "params.tank_design_temperature_c")?),
    };
    // Issue #375: the head type; left out, hemispherical heads (the old way).
    let head_type = match p.get("tank_head_type") {
        None | Some(Value::Null) => b::DEFAULT_TANK_HEAD_TYPE.to_string(),
        Some(Value::String(s)) if s == "hemisphere" || s == "ellipsoidal_2to1" => s.clone(),
        Some(other) => {
            return bad(format!(
                "`params.tank_head_type` must be one of hemisphere, ellipsoidal_2to1, not {}",
                py_repr(other)
            ));
        }
    };
    Ok(b::TankDesign {
        head_type,
        liquid_pressure,
        design_temperature_c,
        design_pressure_mpa: get_or(
            p,
            "tank_design_pressure_mpa",
            b::DEFAULT_TANK_DESIGN_PRESSURE_MPA,
        )?,
        material,
        allowable_stress_mpa: allowable,
        insulation_thickness_mm: get_or(
            p,
            "tank_insulation_thickness_mm",
            b::DEFAULT_TANK_INSULATION_MM,
        )?,
        structure_factor: get_or(p, "tank_structure_factor", b::DEFAULT_TANK_STRUCTURE_FACTOR)?,
    })
}

/// The stress the engine used for `material` before the IGC check (issue #313), when it differs from
/// the material's own; an unknown material is left for the core to refuse.
fn published_design_stress(material: &str) -> Option<f64> {
    ovcd_core::pressure_vessel::get_tank_material(material)
        .ok()
        .filter(|m| m.published_design_stress != m.allowable_stress)
        .map(|m| m.published_design_stress)
}

/// How an LPG Cargo is loaded (issue #315): the fill handed to the engine (the loading limit), the
/// density at the loading temperature, and what they came from, for `drawing.cargo_loading` (not a
/// result, not hashed).
struct Loading {
    fill: f64,
    density_kg_m3: Option<f64>,
    report: Option<Value>,
}

/// The IGC 15.1.4 air temperature of the ambient design conditions (C): the reference temperature
/// of a fully pressurised tank under IGC 15.5.2.
const AMBIENT_DESIGN_AIR_C: f64 = 45.0;

/// The loading of an LPG Cargo (issue #315). `lpg_loading_limit_basis` left out, or
/// `refrigerated_density`, keeps what the engine did before: the cargo data's density, loaded to the
/// fill ratio. Otherwise the density is the saturated liquid density at the loading temperature,
/// and the tank is loaded to LL = FL x rho_R / rho_L (IGC 15.5.1), not above FL. The reference
/// temperature is IGC 15.1.3: fully pressurised, the temperature at which the vapour pressure is the
/// MARVS (15.1.3.1) or, on `igc_15_5_2`, 45 C; otherwise the loading temperature (15.1.3.2).
fn lpg_loading(p: &Map<String, Value>, cargo: &str, fill: f64) -> R<Loading> {
    use ovcd_core::lpg_saturation as sat;
    let old = Loading {
        fill,
        density_kg_m3: None,
        report: None,
    };
    let basis = match p.get("lpg_loading_limit_basis") {
        None | Some(Value::Null) => return Ok(old),
        Some(Value::String(s)) if crate::vdesign::LOADING_LIMIT_BASES.contains(&s.as_str()) => {
            s.as_str()
        }
        Some(other) => {
            return bad(format!(
                "`params.lpg_loading_limit_basis` must be one of {}, not {}",
                crate::vdesign::LOADING_LIMIT_BASES.join(", "),
                py_repr(other)
            ));
        }
    };
    if basis == "refrigerated_density" || !sat_cargo(cargo) {
        return Ok(old);
    }
    let carriage = match p.get("lpg_carriage_condition") {
        Some(Value::String(s)) => s.as_str(),
        _ => "fully_pressurised",
    };
    let t_load = get_or(p, "lpg_loading_temperature_c", 15.0)?;
    let rho_l = match sat::liquid_density_kg_m3(cargo, t_load) {
        Some(v) => v,
        None => {
            return bad(format!(
                "`params.lpg_loading_temperature_c` = {} is outside the saturation table (-50 to \
                 80 C)",
                py_str(t_load)
            ));
        }
    };
    let (t_ref, rule) = if carriage != "fully_pressurised" {
        (
            t_load,
            "IGC 15.1.3.2: the loading temperature (temperature control)",
        )
    } else if basis == "igc_15_5_2" {
        (
            AMBIENT_DESIGN_AIR_C,
            "IGC 15.5.2: 45 C, the air temperature of the ambient design conditions (15.1.4)",
        )
    } else {
        let marvs_bar = get_or(
            p,
            "tank_design_pressure_mpa",
            ovcd_core::barge_capacity::DEFAULT_TANK_DESIGN_PRESSURE_MPA,
        )? * 10.0
            + sat::ATMOSPHERE_BAR;
        match sat::saturation_temperature_c(cargo, marvs_bar) {
            Some(t) => (
                t,
                "IGC 15.1.3.1: the temperature at which the vapour pressure is the MARVS",
            ),
            None => {
                return bad(format!(
                    "the MARVS of {} bar absolute is outside the saturation table of {cargo}",
                    py_str(marvs_bar)
                ));
            }
        }
    };
    let rho_r = sat::liquid_density_kg_m3(cargo, t_ref).ok_or_else(|| {
        Stop::Eval(EvalError::BadInput(format!(
            "the reference temperature {} C is outside the saturation table of {cargo}",
            py_str(t_ref)
        )))
    })?;
    // With temperature control rho_R is rho_L, so LL is FL; it is taken as FL, not worked out.
    let formula = sat::loading_limit(fill, rho_r, rho_l);
    let ll = if t_ref == t_load || formula > fill {
        fill
    } else {
        formula
    };
    Ok(Loading {
        fill: ll,
        density_kg_m3: Some(rho_l),
        report: Some(json!({
            "basis": basis,
            "filling_limit": num(fill),
            "loading_temperature_c": num(t_load),
            "loading_density_kg_m3": num(rho_l),
            "reference_temperature_c": num(t_ref),
            "reference_rule": rule,
            "reference_density_kg_m3": num(rho_r),
            "loading_limit": num(ll),
            "loading_limit_formula": "LL = FL x rhoR / rhoL (IGC 15.5.1), not above FL",
            "cargo_kg_per_m3_of_tank": num(ll * rho_l),
        })),
    })
}

/// Whether the saturation tables hold `cargo` (the LPG Cargos).
fn sat_cargo(cargo: &str) -> bool {
    ovcd_core::lpg_saturation::liquid_density_kg_m3(cargo, 0.0).is_some()
}

/// What the tank was designed to, for the `drawing.tank` object (not a result, not hashed): the
/// values in force, where the wall thickness came from, and the diameter datum.
///
/// Datum (issue #292): `tank_outer_diameter_m` of the results is the outside of the insulation.
/// The side and bottom clearances, the tank top height and the drawing all use that face, and the
/// IGC distances d are measured plate to plate, so the insulation is taken off them (IGC 2.4.2).
/// The steel diameter is that less twice the insulation; the wall thickness formula and the volume
/// both start from the steel diameter. `tank_steel_outer_diameter_m` shows it.
fn tank_report(
    p: &Map<String, Value>,
    inp: &BargeCapacityInputs,
    r: &BargeCapacityResult,
) -> Value {
    let design = &inp.tank_design;
    let steel = r.tank_outer_diameter_m.map(|od| {
        od - 2.0
            * r.insulation_thickness_mm
                .unwrap_or(design.insulation_thickness_mm)
            / 1000.0
    });
    let head = r.gas_basis.as_ref().map(|b| b.tanks.head_wall_thickness);
    // The design liquid pressure the wall took (issue #374): Pgd = a x Z x rho / 1.02e5, with Z the
    // steel outer diameter, and Peq = Po + Pgd.
    let liquid = ovcd_core::barge_capacity::liquid_pressure_basis(
        inp,
        r.bow_void_length_m.unwrap_or(0.0),
        r.stern_void_length_m.unwrap_or(0.0),
        ovcd_core::models::BargeInputs::new(1.0, 1.0, 1.0, 1.0, 1.0).tank_gap_longitudinal,
    )
    .unwrap_or((None, 0.0));
    let pgd = ovcd_core::tanks::liquid_pressure_mpa(liquid.0, steel.unwrap_or(0.0), liquid.1);
    // The dome in force (issue #316): the rule of `dome_dimensions` on the steel diameter, or the
    // typed size.
    let dome = steel.map(|sd| {
        ovcd_core::barge_capacity::dome_dimensions(
            sd,
            p.get("tank_dome_diameter_m").and_then(Value::as_f64).or(
                if p.contains_key("tank_dome_diameter_m") {
                    None
                } else {
                    Some(ovcd_core::barge_capacity::DEFAULT_TANK_DOME_DIAMETER_M)
                },
            ),
            p.get("tank_dome_height_m").and_then(Value::as_f64).or(
                if p.contains_key("tank_dome_height_m") {
                    None
                } else {
                    Some(ovcd_core::barge_capacity::DEFAULT_TANK_DOME_HEIGHT_M)
                },
            ),
        )
    });
    let allowable = design.allowable_stress_mpa.or_else(|| {
        ovcd_core::pressure_vessel::get_tank_material(&design.material)
            .ok()
            .map(|m| m.allowable_stress)
    });
    let text = |key: &str| p.get(key).cloned().unwrap_or(Value::Null);
    Value::Object(
        [
            ("lpg_carriage_condition", text("lpg_carriage_condition")),
            ("design_pressure_mpa", num(design.design_pressure_mpa)),
            ("liquid_pressure", text("tank_liquid_pressure")),
            ("liquid_pressure_accel", opt_num(liquid.0)),
            ("liquid_pressure_density_kg_m3", num(liquid.1)),
            ("liquid_pressure_mpa", num(pgd)),
            (
                "internal_pressure_mpa",
                num(design.design_pressure_mpa + pgd),
            ),
            ("design_temperature_c", text("tank_design_temperature_c")),
            ("material", json!(design.material)),
            ("head_type", json!(design.head_type)),
            ("allowable_stress_mpa", opt_num(allowable)),
            (
                "allowable_stress_basis",
                text("tank_allowable_stress_basis"),
            ),
            ("insulation_type", text("tank_insulation_type")),
            (
                "insulation_thickness_mm",
                num(design.insulation_thickness_mm),
            ),
            ("structure_factor", num(design.structure_factor)),
            (
                "wall_thickness_source",
                json!(if inp.tank_wall_thickness.is_some() {
                    "override"
                } else {
                    "formula"
                }),
            ),
            (
                "cylinder_wall_thickness_mm",
                opt_num(r.tank_wall_thickness_mm),
            ),
            ("head_wall_thickness_mm", opt_num(head)),
            ("tank_outer_diameter_m", opt_num(r.tank_outer_diameter_m)),
            ("tank_steel_outer_diameter_m", opt_num(steel)),
            ("tank_inner_diameter_m", opt_num(r.tank_inner_diameter_m)),
            ("dome_diameter_m", opt_num(dome.map(|d| d.0))),
            ("dome_height_m", opt_num(dome.map(|d| d.1))),
            (
                "dome_source",
                json!(match (
                    p.get("tank_dome_diameter_m").is_some_and(Value::is_null),
                    p.get("tank_dome_height_m").is_some_and(Value::is_null),
                ) {
                    (true, true) => "rule",
                    (false, false) => "override",
                    _ => "rule and override",
                }),
            ),
            (
                "diameter_datum",
                json!(
                    "tank_outer_diameter_m is the outside of the insulation; the clearances, the \
                     tank top and the drawing use that face. The steel outer diameter is that \
                     less twice the insulation; the wall thickness and the volume start from it"
                ),
            ),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect(),
    )
}

/// The typed values of one run that the Rule Trace reads (spec 0004).
pub use ovcd_core::measured::Measured;

fn simulate(p: &Map<String, Value>) -> R<Value> {
    simulate_measured(p, &mut Measured::default(), &RunRules::default())
}

/// The Rule Sets of a run and the Option's inputs, which the gas arrangement search reads (spec
/// 0004, #277). Empty: the search judges by its own check list, as the Python API does.
#[derive(Debug, Clone, Default)]
pub struct RunRules {
    pub sets: Vec<ovcd_core::rules::RuleSet>,
    pub inputs: ga::RuleInputs,
}

fn simulate_measured(
    p: &Map<String, Value>,
    measured: &mut Measured,
    rules: &RunRules,
) -> R<Value> {
    let vessel_id = match p.get("vessel") {
        None => "box_barge",
        Some(Value::String(s)) => s.as_str(),
        Some(_) => return bad("`params.vessel` must be text"),
    };
    let (label, hold_type, phase) = vessel(vessel_id)?;
    let cargo_key = match p.get("cargo") {
        Some(Value::String(s)) => s.as_str(),
        _ => return bad("`params.cargo` must be a cargo key"),
    };
    let cargo = cargo_entry(cargo_key)?;
    if cargo.phase != phase {
        return Ok(error_result(format!(
            "{} cannot be carried in a {}.",
            cargo.name,
            label.to_lowercase()
        )));
    }
    let (l, b, d) = (need(p, "loa_m")?, need(p, "beam_m")?, need(p, "depth_m")?);
    // One water density for the whole run: the draught, the stability and the resistance curves.
    let rho = get_or(p, "water_density_t_per_m3", gs::DEFAULT_WATER_DENSITY)?;
    let self_propelled = p.get("self_propelled").is_some_and(truthy);
    // Issue #350: the Stern Form of a self-propelled hull, at the design draught, and the block
    // coefficient: as given, or (null) the raked box with the Stern Form.
    let t_design = design_draught(p, d)?;
    let stern_geometry = stern_of(p, (l, b, d), t_design, self_propelled)?;
    let stern = stern_geometry.as_ref();
    let cb = cb_of(p, stern, (l, b, d), t_design)?;
    if vessel_id == "gas_barge" {
        return simulate_gas(
            p,
            label,
            cargo,
            cargo_key,
            (l, b, d),
            (cb, rho),
            (self_propelled, stern),
            rules,
        )
        .map(|(v, gas)| {
            *measured = gas;
            v
        });
    }
    let freeboard = get_or(p, "freeboard_min_m", 0.3)?;
    let air_draft_limit = match p.get("air_draft_limit_m") {
        None | Some(Value::Null) => None,
        Some(v) => Some(py_float(v, "params.air_draft_limit_m")?),
    };
    let mut inp = BargeCapacityInputs::new(l, b, d);
    inp.cargo_type = cargo_key.to_string();
    inp.self_propelled = self_propelled;
    inp.block_coefficient = cb;
    inp.water_density = rho;
    inp.required_freeboard = freeboard;
    inp.air_draft_limit_m = air_draft_limit;
    // The end spaces. A key left out keeps the core default, the value the engine always used.
    inp.bow_rake_to_depth_ratio = get_or(
        p,
        "bow_rake_length_to_depth_ratio",
        inp.bow_rake_to_depth_ratio,
    )?;
    inp.pushed_stern_void_m = get_or(p, "pushed_stern_void_m", inp.pushed_stern_void_m)?;
    inp.bulk_end_bulkhead_m = get_or(p, "bulk_end_bulkhead_m", inp.bulk_end_bulkhead_m)?;
    inp.aft_design = aft_design_of(p)?;
    inp.stern_block_mass = stern_mass_of(p)?;
    inp.stern_geometry = stern.cloned();
    inp.machinery = machinery_of(p, stern, (l, b, d), t_design, self_propelled)?;
    // The hold design values. A key left out keeps the core default, which is the value the engine
    // used before these were inputs.
    inp.wing_void_width = get_or(p, "hold_wing_void_width_m", inp.wing_void_width)?;
    inp.double_bottom_height_bulk =
        get_or(p, "hold_double_bottom_m", inp.double_bottom_height_bulk)?;
    inp.hopper_slope_h_per_v = get_or(p, "hopper_slope_h_per_v", inp.hopper_slope_h_per_v)?;
    inp.coaming_height = get_or(p, "hatch_coaming_height_m", inp.coaming_height)?;
    inp.n_holds = whole_number(get_or(p, "n_holds", inp.n_holds as f64)?, "n_holds")?;
    inp.hold_fill_ratio = get_or(p, "hold_fill_ratio", inp.hold_fill_ratio)?;
    inp.inner_bottom_t_mm = get_or(p, "inner_bottom_plate_mm", inp.inner_bottom_t_mm)?;
    inp.hopper_plate_t_mm = get_or(p, "hopper_plate_mm", inp.hopper_plate_t_mm)?;
    inp.bulkhead_t_mm = get_or(p, "hold_bulkhead_plate_mm", inp.bulkhead_t_mm)?;
    inp.coaming_t_mm = get_or(p, "hatch_coaming_plate_mm", inp.coaming_t_mm)?;
    if let Some(h) = hold_type {
        inp.hold_type = h.to_string();
        if h != "sloped_hopper" {
            inp.lightship_k_t_per_m3 = Some(need(p, "lightship_k_t_per_m3")?);
        }
    }
    let r = match barge_capacity::compute(&inp) {
        Ok(r) => r,
        Err(e) if e.kind == "ValueError" => return Ok(error_result(e.message)),
        Err(e) => return Err(Stop::Raised(e.kind.to_string(), e.message)),
    };

    // --- Draught limit and the weight-bound cargo ---
    let ch = channel(p)?;
    let hull_limit = format!("Hull (depth {MINUS} min. freeboard)");
    let channel_name = format!("Channel (depth {MINUS} under-keel clearance)");
    let mut limits: Vec<(&str, f64)> = vec![(hull_limit.as_str(), d - freeboard)];
    if let Some(v) = truthy_float(p, "draught_limit_m")? {
        limits.push(("Draught limit", v));
    }
    if let Some(v) = channel_limit(&ch)? {
        limits.push((channel_name.as_str(), v));
    }
    let (lim_name, t_max) = min_item(&limits);
    let ls = r.lightship_t;
    let disp_max = cb * l * b * t_max * rho;
    let cargo_weight_bound = py_max(0.0, disp_max - ls);
    let cargo_volume_bound = r.cargo_capacity_t;
    let cargo_t = py_min(cargo_volume_bound, cargo_weight_bound);
    let mut ids = TextIds::default();
    let governs = if cargo_volume_bound <= cargo_weight_bound {
        Text::plain("hold volume")
    } else {
        governs_draught(&lim_name)
    };
    ids.text(&governs);
    let disp = ls + cargo_t;
    let deadweight = disp - ls;
    measured.deadweight_t = Some(deadweight);
    let t = py_div(disp, cb * l * b * rho)?;
    let h_top = r.air_draft_top_above_keel_m;

    let mut checks = vec![json!({
        "name": "Cargo > 0",
        "ok": cargo_t > 0.0,
        "message": if cargo_t <= 0.0 { "Lightship alone exceeds the allowed draught." } else { "OK" },
    })];
    capacity_checks(&r, &mut checks);
    ids.checks(&r.checks);
    ids.texts(&r.warnings);
    if let Some(limit) = truthy_float(p, "air_draft_limit_m")? {
        checks.push(air_draft_check(&r, limit, &mut ids)?);
    }
    let curve_list = curves(
        p,
        l,
        b,
        t,
        (cb, stern.map_or(0.0, |g| g.extra_wetted_surface(t))),
        disp,
        rho,
    )?;
    let geo = json!({
        "bow_void_length_m": opt_num(r.bow_void_length_m),
        "stern_void_length_m": opt_num(r.stern_void_length_m),
        "cargo_zone_length_m": opt_num(r.cargo_zone_length_m),
        "bow_rake_length_m": opt_num(r.bow_rake_length_m),
        "tank_outer_diameter_m": opt_num(r.tank_outer_diameter_m),
        "n_tanks_wide": opt_int(r.n_tanks_wide),
        "n_tanks_total": opt_int(r.n_tanks_total),
        "side_clearance_m": opt_num(r.side_clearance_m),
        "bottom_clearance_m": opt_num(r.bottom_clearance_m),
        "hold_height_m": opt_num(r.hold_height_m),
        "hold_top_width_m": opt_num(r.hold_top_width_m),
        "hold_bottom_width_m": opt_num(r.hold_bottom_width_m),
        "insulation_thickness_mm": opt_num(r.insulation_thickness_mm),
    });
    let geo = with_stern_geometry(geo, stern);
    let mut dry_parts = structure_json(&r, inp.separation.as_ref(), Some(&inp));
    if let Some(g) = stern {
        dry_parts.push(("stern", stern_drawing(g)));
    }
    let drawing = drawing_json(dry_parts);
    with_text_ids(
        with_stern_mass(
            json!({
                "vessel": label,
                "cargo": cargo.name,
                "phase": cargo.phase.key(),
                "self_propelled": self_propelled,
                "arrangement": Value::Null,
                "dims": { "loa_m": num(l), "beam_m": num(b), "depth_m": num(d), "cb": num(cb) },
                "geometry": geo,
                "drawing": drawing,
                "results": {
                    "cargo_t": num(cargo_t),
                    "cargo_volume_bound_t": num(cargo_volume_bound),
                    "cargo_weight_bound_t": num(cargo_weight_bound),
                    "governs": governs,
                    "cargo_volume_m3": num(r.cargo_volume_m3),
                    "lightship_t": num(ls),
                    "displacement_t": num(disp),
                    "deadweight_t": num(deadweight),
                    "draught_m": num(t),
                    "draught_max_m": num(t_max),
                    "freeboard_m": num(d - t),
                    "light_draught_m": opt_num(r.light_draft_m),
                    "air_draft_loaded_m": num(h_top - t),
                    "air_draft_light_m": opt_num(r.air_draft_light_m),
                    "h_over_T": h_over_t(&ch, t)?,
                },
                "checks": checks,
                "warnings": r.warnings,
                "curves": curve_list,
                "cost_masses": Value::Null,
            }),
            (&r.stern_block_items, &r.trim_conditions),
        ),
        ids,
    )
}

/// "draught — <limit>": the draught governs the cargo, and the limit sets the draught.
fn governs_draught(limit: &str) -> Text {
    text!(
        "name-governs-draught",
        "draught \u{2014} {limit}",
        limit = limit
    )
}

/// The result with its `text_ids` (ADR 0014): a reading aid, outside the results hash.
/// A condition with its trim (issue #349), only when the trim was worked out: a file without the
/// stern block mass gets no new field.
fn with_trim(mut c: Value, trim: Option<&ovcd_core::models::TrimResult>) -> Value {
    if let Some(t) = trim {
        c["trim_m"] = num(t.trim);
        c["draught_aft_m"] = num(t.draft_aft);
        c["draught_fwd_m"] = num(t.draft_fwd);
    }
    c
}

/// The design draught (m): the least of the hull's depth less the freeboard, the draught limit and
/// the channel's depth less its under-keel clearance, as the draught cap is worked out below.
fn design_draught(p: &Map<String, Value>, d: f64) -> R<f64> {
    let mut t = d - get_or(p, "freeboard_min_m", 0.3)?;
    if let Some(v) = truthy_float(p, "draught_limit_m")?
        && v < t
    {
        t = v;
    }
    if let Some(v) = channel_limit(&channel(p)?)?
        && v < t
    {
        t = v;
    }
    Ok(t)
}

/// The Stern Form of a self-propelled hull (issue #350), `None` for any other hull, for params with
/// no Stern Form, and for today's square stern (a run of 0).
fn stern_of(
    p: &Map<String, Value>,
    (l, b, d): (f64, f64, f64),
    t: f64,
    self_propelled: bool,
) -> R<Option<ovcd_core::stern_form::SternGeometry>> {
    use ovcd_core::stern_form::{HullDims, SternForm, SternFormInputs, geometry};
    if !self_propelled {
        return Ok(None);
    }
    let form = match p.get("stern_form") {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::String(s)) => match SternForm::from_key(s) {
            Some(f) => f,
            None => {
                return bad(format!(
                    "`params.stern_form` {} is not a Stern Form",
                    py_repr(&json!(s))
                ));
            }
        },
        Some(other) => {
            return bad(format!(
                "`params.stern_form` must be text, not {}",
                py_repr(other)
            ));
        }
    };
    let opt = |k: &str| -> R<Option<f64>> {
        match p.get(k) {
            None | Some(Value::Null) => Ok(None),
            Some(v) => Ok(Some(py_float(v, &format!("params.{k}"))?)),
        }
    };
    let propulsor = p
        .get("propulsor_type")
        .and_then(Value::as_str)
        .unwrap_or("unspecified");
    let count = if propulsor == "unspecified" {
        get_or(p, "thruster_count", 3.0)?
    } else {
        get_or(p, "propulsor_count", 2.0)?
    };
    let g = geometry(
        &SternFormInputs {
            form,
            run_length_m: opt("stern_run_length_m")?,
            transom_height_m: opt("stern_transom_height_m")?,
            tunnel_crown_height_m: opt("tunnel_crown_height_m")?,
            propulsor_count: whole_number(count, "propulsor_count")?.max(1) as u32,
            propeller_diameter_m: opt("propeller_diameter_m")?,
            max_unit_diameter_m: ovcd_core::propulsor::PropulsorType::from_key(propulsor)
                .and_then(|t| t.max_unit_diameter_m()),
            parallel_shafts: propulsor.starts_with("shaft"),
        },
        &HullDims {
            loa: l,
            beam: b,
            depth: d,
            design_draught: t,
        },
    );
    Ok(if g.is_legacy() { None } else { Some(g) })
}

/// The machinery of a specified Propulsor (issue #352), or `None` for an `unspecified` one (today's
/// stern block) and for a hull with no propulsion of its own.
fn machinery_of(
    p: &Map<String, Value>,
    stern: Option<&ovcd_core::stern_form::SternGeometry>,
    (l, b, d): (f64, f64, f64),
    t: f64,
    self_propelled: bool,
) -> R<Option<ovcd_core::machinery::MachineryInputs>> {
    use ovcd_core::machinery::{MachineryInputs, PowerSource, default_bow_thruster_kw};
    let key = p
        .get("propulsor_type")
        .and_then(Value::as_str)
        .unwrap_or("unspecified");
    let (true, Some(propulsor)) = (
        self_propelled,
        ovcd_core::propulsor::PropulsorType::from_key(key),
    ) else {
        return Ok(None);
    };
    let source = p
        .get("power_source")
        .and_then(Value::as_str)
        .unwrap_or("battery_electric");
    let Some(power_source) = PowerSource::from_key(source) else {
        return bad(format!(
            "`params.power_source` {} is not a Power Source",
            py_repr(&json!(source))
        ));
    };
    let count = whole_number(get_or(p, "propulsor_count", 2.0)?, "propulsor_count")?.max(1) as u32;
    let diameter = match stern {
        Some(g) => g.propeller_diameter_m,
        None => match truthy_float(p, "propeller_diameter_m")? {
            Some(dia) => dia,
            None => {
                let d_max = ovcd_core::stern_form::K_D_OPEN * t;
                propulsor
                    .max_unit_diameter_m()
                    .map_or(d_max, |m| d_max.min(m))
            }
        },
    };
    let bow_thruster = p
        .get("bow_thruster")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let bow_thruster_kw = if bow_thruster == "none" {
        None
    } else {
        Some(match truthy_float(p, "bow_thruster_power_kw")? {
            Some(kw) => kw,
            None => default_bow_thruster_kw(l, b, t),
        })
    };
    let rake = get_or(
        p,
        "bow_rake_length_to_depth_ratio",
        ovcd_core::barge_capacity::DEFAULT_BOW_RAKE_TO_DEPTH_RATIO,
    )? * d;
    Ok(Some(MachineryInputs {
        power_source,
        propulsor,
        propulsor_count: count,
        propeller_diameter_m: diameter,
        installed_power_kw: truthy_float(p, "installed_power_kw")?,
        battery_energy_kwh: truthy_float(p, "battery_energy_kwh")?,
        bow_thruster_kw,
        // The bow thruster sits in the bow, aft of the rake.
        bow_thruster_lcg_m: l - rake,
        stern_run_m: stern.map_or(0.0, |g| g.run_m),
    }))
}

/// The block coefficient (issue #350): as the params give it; null asks for the rule, the volume
/// of the raked box (today's bow rake and the Stern Form) at the design draught over L x B x T,
/// or 0.85 with no Stern Form, the value the engine always used.
fn cb_of(
    p: &Map<String, Value>,
    stern: Option<&ovcd_core::stern_form::SternGeometry>,
    (l, b, d): (f64, f64, f64),
    t: f64,
) -> R<f64> {
    match p.get("cb") {
        Some(Value::Null) => match stern {
            Some(g) if t > 0.0 => {
                let rake = get_or(
                    p,
                    "bow_rake_length_to_depth_ratio",
                    ovcd_core::barge_capacity::DEFAULT_BOW_RAKE_TO_DEPTH_RATIO,
                )? * d;
                let hull = ovcd_core::hull::HullGeometry::new(l, b, d, rake, d, 0.0, 0.0);
                Ok((hull.displaced_volume(t) - g.removed(t).0) / (l * b * t))
            }
            _ => Ok(DEFAULT_BLOCK_COEFFICIENT),
        },
        _ => need(p, "cb"),
    }
}

/// The block coefficient the engine used before it had a rule.
const DEFAULT_BLOCK_COEFFICIENT: f64 = 0.85;

/// The Stern Form in the results' `geometry` (issue #350), only when there is one: a square stern
/// adds no field. `stern_rake_length_m` is the run of a transom or pram stern, which the drawings
/// read; a tunnel stern keeps its bottom on the base, so it has none.
fn with_stern_geometry(
    mut geo: Value,
    stern: Option<&ovcd_core::stern_form::SternGeometry>,
) -> Value {
    if let Some(g) = stern {
        let form = match g.form {
            ovcd_core::stern_form::SternForm::Transom => "transom",
            ovcd_core::stern_form::SternForm::Pram => "pram",
            ovcd_core::stern_form::SternForm::Tunnel => "tunnel",
        };
        geo["stern_form"] = json!(form);
        if g.form != ovcd_core::stern_form::SternForm::Tunnel {
            geo["stern_rake_length_m"] = num(g.run_m);
        }
        geo["stern_run_length_m"] = num(g.run_m);
        geo["stern_transom_height_m"] = num(g.transom_height_m);
        geo["propeller_diameter_m"] = num(g.propeller_diameter_m);
        geo["propeller_d_max_m"] = num(g.d_max_m);
    }
    geo
}

/// The Stern Form's outlines for the drawings (not a result): the bottom profile and the tunnel
/// sections at the transom.
fn stern_drawing(g: &ovcd_core::stern_form::SternGeometry) -> Value {
    let pts = |v: &[[f64; 2]]| {
        v.iter()
            .map(|p| json!([num(p[0]), num(p[1])]))
            .collect::<Vec<_>>()
    };
    json!({
        "profile": pts(&g.profile()),
        "tunnel_sections": g.tunnel_sections().iter().map(|s| pts(s)).collect::<Vec<_>>(),
        "tunnel_crown_height_m": num(g.tunnel_crown_height_m),
    })
}

/// Whether the stern block's mass is in the lightship (issue #348), and what it needs. A key left
/// out keeps it out, as before.
fn stern_mass_of(p: &Map<String, Value>) -> R<Option<ovcd_core::aft_arrangement::SternMassInputs>> {
    if !p.get("stern_block_mass_included").is_some_and(truthy) {
        return Ok(None);
    }
    Ok(Some(ovcd_core::aft_arrangement::SternMassInputs {
        battery_unit_energy_kwh: get_or(p, "battery_unit_energy_kwh", 5000.0)?,
        installed_power_kw: truthy_float(p, "installed_power_kw")?,
    }))
}

/// The stern block's mass items in the results (issue #348), only when they are in the lightship:
/// a file without the stern mass gets no new field, so its results hash is unchanged.
fn with_stern_mass(
    mut result: Value,
    (items, trims): (
        &[ovcd_core::weights::MassItem],
        &[(&str, ovcd_core::models::TrimResult)],
    ),
) -> Value {
    // Issue #349: a dry-bulk hull's trim, laden and light, beside the block.
    if !trims.is_empty() {
        result["trim"] = Value::Array(
            trims
                .iter()
                .map(|(name, t)| {
                    with_trim(
                        json!({ "name": name, "draught_m": num(t.draft_aft + t.trim / 2.0) }),
                        Some(t),
                    )
                })
                .collect(),
        );
    }
    if !items.is_empty() {
        result["stern_block_mass"] = json!({
            "total_t": num(ovcd_core::weights::total_mass(items)),
            "items": items.iter().map(|i| json!({
                "label": i.label,
                "mass_t": num(i.mass_t),
                "lcg_m": num(i.lcg_m),
                "kcg_m": num(i.kcg_m),
            })).collect::<Vec<_>>(),
        });
    }
    result
}

fn with_text_ids(mut result: Value, ids: TextIds) -> R<Value> {
    result["text_ids"] = ids.finish(&result);
    Ok(result)
}

/// Tank filling limit by ruleset: IGC 15.3 default 98 %, for both IGC rulesets.
fn fill_by_ruleset(_ruleset: &str) -> f64 {
    0.98
}

#[allow(clippy::too_many_arguments)] // one call site; the run's Rule Sets join the simulate params
fn simulate_gas(
    p: &Map<String, Value>,
    label: &str,
    cargo: &CargoProperties,
    cargo_key: &str,
    (l, b, d0): (f64, f64, f64),
    (cb, rho): (f64, f64),
    (self_propelled, stern): (bool, Option<&ovcd_core::stern_form::SternGeometry>),
    rules: &RunRules,
) -> R<(Value, Measured)> {
    let ruleset = match p.get("ruleset") {
        None => "IGC_2G",
        Some(Value::String(s)) => s.as_str(),
        // Python tests `ruleset not in RULESETS`, so any other value is an unknown ruleset.
        Some(other) => {
            return Ok((
                error_result(format!("Unknown ruleset {}.", py_repr(other))),
                Measured::default(),
            ));
        }
    };
    if !RULESETS.contains(&ruleset) {
        return Ok((
            error_result(format!("Unknown ruleset {}.", repr_str(ruleset))),
            Measured::default(),
        ));
    }
    let fill = match truthy_float(p, "fill_ratio")? {
        Some(v) => v,
        None => fill_by_ruleset(ruleset),
    };
    let loading = lpg_loading(p, cargo_key, fill)?;
    let fb = get_or(p, "freeboard_min_m", 0.3)?;
    let ch = channel(p)?;
    let channel_name = format!("Channel (depth {MINUS} under-keel clearance)");
    let hull_limit = format!("Hull (depth {MINUS} min. freeboard)");
    let mut limits: Vec<(&str, f64)> = Vec::new();
    if let Some(v) = truthy_float(p, "draught_limit_m")? {
        limits.push(("Draught limit", v));
    }
    if let Some(v) = channel_limit(&ch)? {
        limits.push((channel_name.as_str(), v));
    }
    let optimise = p.get("optimise_depth").is_some_and(truthy);
    let depths: Vec<f64> = if optimise {
        let cap = if limits.is_empty() {
            None
        } else {
            let mut m = limits[0].1;
            for &(_, v) in &limits[1..] {
                if v < m {
                    m = v;
                }
            }
            Some(m)
        };
        let picked: Vec<f64> = DEFAULT_DEPTHS_M
            .iter()
            .copied()
            .filter(|dd| cap.is_none_or(|c| dd - fb >= c))
            .collect();
        if picked.is_empty() {
            vec![DEFAULT_DEPTHS_M[DEFAULT_DEPTHS_M.len() - 1]]
        } else {
            picked
        }
    } else {
        limits.push((hull_limit.as_str(), d0 - fb));
        vec![d0]
    };
    if limits.is_empty() {
        // Optimising depth with no waterway limit: the deepest hull sets it.
        limits.push((hull_limit.as_str(), depths[depths.len() - 1] - fb));
    }
    let (lim_name, t_max) = min_item(&limits);
    let options = FitOptions {
        ruleset: ruleset.to_string(),
        fill_ratio: loading.fill,
        cargo_density_kg_m3: loading.density_kg_m3,
        self_propelled,
        depths: depths.clone(),
        block_coefficient: cb,
        max_tank_od_m: get_or(
            p,
            "max_tank_od_m",
            ovcd_core::barge_capacity::DEFAULT_MAX_TANK_OD_M,
        )?,
        bow_rake_to_depth_ratio: get_or(
            p,
            "bow_rake_length_to_depth_ratio",
            ovcd_core::barge_capacity::DEFAULT_BOW_RAKE_TO_DEPTH_RATIO,
        )?,
        pushed_stern_void_m: get_or(
            p,
            "pushed_stern_void_m",
            ovcd_core::barge_capacity::DEFAULT_PUSHED_STERN_VOID_M,
        )?,
        tanks_in_line: whole_number(
            get_or(
                p,
                "tanks_in_line",
                ovcd_core::barge_capacity::DEFAULT_TANKS_IN_LINE as f64,
            )?,
            "tanks_in_line",
        )?,
        tank_support_height_m: get_or(
            p,
            "tank_support_height_m",
            ovcd_core::barge_capacity::DEFAULT_TANK_SUPPORT_HEIGHT_M,
        )?,
        tank_dome_height_m: dome_of(p, "tank_dome_height_m")?.unwrap_or(0.0),
        tank_dome_diameter_m: dome_of(p, "tank_dome_diameter_m")?.unwrap_or(0.0),
        tank_dome_height_by_rule: dome_of(p, "tank_dome_height_m")?.is_none(),
        tank_dome_diameter_by_rule: dome_of(p, "tank_dome_diameter_m")?.is_none(),
        wing_tank_gap_m: get_or(
            p,
            "wing_tank_gap_m",
            ovcd_core::barge_capacity::DEFAULT_WING_TANK_GAP_M,
        )?,
        wing_tank_height_ratio: get_or(
            p,
            "wing_tank_height_ratio",
            ovcd_core::barge_capacity::DEFAULT_WING_TANK_HEIGHT_RATIO,
        )?,
        tank_wall_thickness_mm: tank_wall_of(p)?,
        tank_design: tank_design_of(p)?,
        water_density_t_per_m3: rho,
        required_freeboard_m: fb,
        air_draft_limit_m: truthy_float(p, "air_draft_limit_m")?,
        aft_design: aft_design_of(p)?,
        stern_block_mass: stern_mass_of(p)?,
        stern_geometry: stern.cloned(),
        rules: rules.sets.clone(),
        rule_inputs: rules.inputs.clone(),
        machinery: machinery_of(p, stern, (l, b, d0), design_draught(p, d0)?, self_propelled)?,
        ..FitOptions::default()
    };
    let f = ga::fit(l, b, cargo_key, t_max, &options)
        .map_err(|e| Stop::Raised(e.kind.to_string(), e.message))?;
    let r = &f.engine_result;
    let d = f.depth_m;
    let t = if f.payload_t > 0.0 {
        f.draft_at_payload_m
    } else {
        // Mutation survivor (#64), the guard `x != 0.0` to `true` or `false`: equivalent. When `x`
        // is 0.0 both arms give 0.0, and the guard only chooses between `x` and 0.0.
        match r.light_draft_m {
            Some(x) if x != 0.0 => x,
            _ => 0.0,
        }
    };
    let disp = r.lightship_t + f.payload_t;
    let mut ids = TextIds::default();
    let governs = match f.binding.as_str() {
        "volume" => Text::plain("tank volume"),
        "draft" => governs_draught(&lim_name),
        "no cargo area" => Text::plain("no cargo area (hull shorter than its fixed end spaces)"),
        other => return bad(format!("unknown binding `{other}`")),
    };
    ids.text(&governs);
    let mut checks = vec![json!({
        "name": "Cargo > 0",
        "ok": f.payload_t > 0.0,
        "message": if f.payload_t > 0.0 { "OK" } else { "Nothing fits: lengthen or widen the hull." },
    })];
    if !f.feasible {
        // Said plainly: no arrangement passes (issue #264). The design shown is the closest one.
        let names: Vec<&str> = f
            .failed_checks
            .iter()
            .map(|m| m.split_once(": ").map_or(m.as_str(), |(name, _)| name))
            .collect();
        let message = text!(
            "check-feasible-arrangement",
            "No arrangement passes every check on this hull. The closest one is shown. It fails: {fails}.",
            fails = names.join("; "),
        );
        ids.text(&message);
        checks.push(json!({
            "name": "Feasible arrangement",
            "ok": false,
            "message": message,
        }));
    }
    capacity_checks(r, &mut checks);
    ids.checks(&r.checks);
    ids.texts(&r.warnings);
    // A tank longer than 7 outer diameters is listed when it fails, as the search names it.
    for msg in &f.failed_checks {
        if let Some(rest) = msg.strip_prefix("tank_length_over_diameter: ") {
            checks.push(json!({
                "name": "tank length over diameter",
                "ok": false,
                "message": rest,
            }));
        }
    }
    if let Some(limit) = truthy_float(p, "air_draft_limit_m")? {
        checks.push(air_draft_check(r, limit, &mut ids)?);
    }
    let mut stability = Value::Null;
    // What a view marks on the GZ curves. Not a result: it is left out of the results hash.
    let mut stability_marks = Value::Null;
    // The loading conditions, typed, for the Rule Trace.
    let mut conditions = Vec::new();
    if f.payload_t > 0.0 && r.gas_basis.is_some() {
        let basis = GasCapacityResult::try_from(r)
            .map_err(|m| Stop::Eval(EvalError::BadInput(m.into())))?;
        let a = gs::assess(&basis, f.payload_t, cb, rho, gs::criteria_set_for(ruleset))
            .map_err(|e| Stop::Raised(e.type_name().to_string(), e.message().to_string()))?;
        for c in &a.conditions {
            let k = ga::stability_check(c);
            ids.checks(core::slice::from_ref(&k));
            checks.push(json!({
                "name": k.name,
                "ok": k.ok,
                "message": k.message,
            }));
        }
        let set_label = match a.criteria_set.as_str() {
            "IS_CODE_2008" => "IMO IS Code 2008, Part A 2.2",
            other => return bad(format!("unknown criteria set `{other}`")),
        };
        stability_marks = json!({ "criteria_angles_deg": gs::CRITERIA_ANGLES_DEG });
        ids.texts(&a.limitations);
        conditions = a.conditions.clone();
        stability = json!({
            "criteria_set": set_label,
            "ok": a.ok(),
            "governing": a.governing().name,
            "limitations": a.limitations,
            "conditions": a
                .conditions
                .iter()
                .enumerate()
                .map(|(i, c)| with_trim(simulate_condition(c), a.trims.get(i)))
                .collect::<Vec<_>>(),
        });
    }
    let curve_list = curves(
        p,
        l,
        b,
        t,
        (cb, stern.map_or(0.0, |g| g.extra_wetted_surface(t))),
        disp,
        rho,
    )?;
    let geo = json!({
        "bow_void_length_m": opt_num(r.bow_void_length_m),
        "stern_void_length_m": opt_num(r.stern_void_length_m),
        "cargo_zone_length_m": opt_num(r.cargo_zone_length_m),
        "bow_rake_length_m": opt_num(r.bow_rake_length_m),
        "tank_outer_diameter_m": opt_num(r.tank_outer_diameter_m),
        "n_tanks_wide": opt_int(r.n_tanks_wide),
        "n_tanks_total": opt_int(r.n_tanks_total),
        "side_clearance_m": opt_num(r.side_clearance_m),
        "bottom_clearance_m": opt_num(r.bottom_clearance_m),
        "insulation_thickness_mm": opt_num(r.insulation_thickness_mm),
    });
    let geo = with_stern_geometry(geo, stern);
    let mut drawing_parts = structure_json(r, f.engine_inputs.separation.as_ref(), None);
    drawing_parts.push(("tank", tank_report(p, &f.engine_inputs, r)));
    if let Some(report) = loading.report {
        drawing_parts.push(("cargo_loading", report));
    }
    if let Some(g) = stern {
        drawing_parts.push(("stern", stern_drawing(g)));
    }
    let drawing = drawing_json(drawing_parts);
    let air_loaded = r.air_draft_top_above_keel_m - t;
    with_text_ids(
        with_stern_mass(
        json!({
            "vessel": label,
            "cargo": cargo.name,
            "phase": cargo.phase.key(),
            "self_propelled": self_propelled,
            "dims": { "loa_m": num(l), "beam_m": num(b), "depth_m": num(d), "cb": num(cb) },
            "geometry": geo,
            "drawing": drawing,
            "arrangement": {
                "ruleset": ruleset,
                "fill_ratio": num(fill),
                "n_wide": f.n_tanks_wide,
                "n_long": f.n_tanks_long,
                "tank_od_m": num(f.tank_od_m),
                "per_tank_m3": num(f.per_tank_volume_m3),
                "side_clearance_m": num(f.side_clearance_m),
                "depth_m": num(f.depth_m),
                "depth_optimised": optimise,
                "candidates_tried": f.candidates_tried,
                "feasible": f.feasible,
            },
            "results": {
                "cargo_t": num(f.payload_t),
                "cargo_volume_bound_t": num(f.volume_capacity_t),
                "cargo_weight_bound_t": num(py_max(0.0, cb * l * b * t_max * rho - r.lightship_t)),
                "governs": governs,
                "cargo_volume_m3": num(r.cargo_volume_m3),
                "lightship_t": num(r.lightship_t),
                "displacement_t": num(disp),
                "deadweight_t": num(f.payload_t),
                "draught_m": num(t),
                "draught_max_m": num(t_max),
                "freeboard_m": num(d - t),
                "light_draught_m": opt_num(r.light_draft_m),
                "air_draft_loaded_m": num(air_loaded),
                "air_draft_light_m": opt_num(r.air_draft_light_m),
                "h_over_T": h_over_t(&ch, t)?,
            },
            "checks": checks,
            "warnings": r.warnings,
            "curves": curve_list,
            "stability": stability,
            "stability_marks": stability_marks,
            "cost_masses": r.gas_basis.as_ref().map_or(Value::Null, |b| crate::cost::gas_masses(&b.weights)),
        }),
            (&r.stern_block_items, &r.trim_conditions),
        ),
        ids,
    )
    .map(|v| {
        (
            v,
            Measured {
                conditions,
                deadweight_t: Some(f.payload_t),
                tank: r.tank_rules.clone(),
                separation: r.separation.clone().map(|s| {
                    let plane = f
                        .engine_inputs
                        .separation
                        .as_ref()
                        .is_some_and(|i| i.plane_tank_ends);
                    (s, plane)
                }),
                checks: Vec::new(),
            },
        )
    })
}

/// One stability condition as the simulator lists it: the fields it keeps, `fill` for the fill
/// fraction, and each criterion without anything extra.
fn simulate_condition(c: &ovcd_core::gas_stability::Condition) -> Value {
    let full = condition_json(c);
    let pick = |k: &str| full.get(k).cloned().unwrap_or(Value::Null);
    json!({
        "name": pick("name"),
        "cargo_t": pick("cargo_t"),
        "fill": pick("fill_fraction"),
        "displacement_t": pick("displacement_t"),
        "draught_m": pick("draught_m"),
        "kg_m": pick("kg_m"),
        "gm_solid_m": pick("gm_solid_m"),
        "free_surface_m": pick("free_surface_m"),
        "gm_fluid_m": pick("gm_fluid_m"),
        "max_gz_m": pick("max_gz_m"),
        "max_gz_deg": pick("max_gz_deg"),
        "deck_edge_deg": pick("deck_edge_deg"),
        "gz": pick("gz"),
        "criteria": pick("criteria"),
    })
}

/// What one `simulate` call came to.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// The result object (which may itself be `{"error": ...}`, as Python returns it).
    Result(Value),
    /// A Python exception escaped `simulate`: class name and message.
    Raised(String, String),
    /// The params are outside what this port models; Python would raise `KeyError` or `TypeError`.
    Refused(String),
}

/// `simulate(params)` as the app calls it: params in, an [`Outcome`] out, with the typed values the
/// Rule Trace reads.
pub(crate) fn simulate_params_measured(
    params: &Map<String, Value>,
    rules: &RunRules,
) -> (Outcome, Measured) {
    let mut measured = Measured::default();
    let outcome = match simulate_measured(params, &mut measured, rules) {
        Ok(v) => Outcome::Result(v),
        Err(Stop::Raised(kind, message)) => Outcome::Raised(kind, message),
        Err(Stop::Eval(e)) => Outcome::Refused(e.to_string()),
    };
    (outcome, measured)
}

/// Run one `simulate` function on case-shaped arguments.
pub fn run(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "cargo_options" => ok(cargo_options()),
        "simulate" => {
            let params = args
                .get("params")
                .and_then(Value::as_object)
                .ok_or_else(|| EvalError::BadInput("`args.params` must be an object".into()))?;
            match simulate(params) {
                Ok(v) => ok(v),
                Err(Stop::Eval(e)) => Err(e),
                Err(Stop::Raised(kind, message)) => raised(&kind, &message),
            }
        }
        other => Err(EvalError::BadInput(format!(
            "simulate has no function `{other}`"
        ))),
    }
}
