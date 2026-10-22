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
use ovcd_core::hydrostatics::{Hull, HullSpec};
use ovcd_core::pyfmt::py_str;
use ovcd_core::pystr::{parse_float, repr_str};
use ovcd_core::resistance::{self, EnvironmentInputs, ResistanceInputs, TugInputs};
use ovcd_core::text;
use ovcd_core::text::{Arg, Text};
use serde_json::{Map, Value, json};

const MINUS: &str = "\u{2212}";

/// Why a simulation stopped before it had a result.
pub(crate) enum Stop {
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

/// A number param that may be left out or null (`None`); 0 is a number.
/// The deck openings of the angle of down-flooding (issue #592); none when the input is null.
fn openings_of(p: &Map<String, Value>) -> R<Vec<ovcd_core::hydrostatics::Opening>> {
    crate::openings::openings_of(p.get("downflooding_openings").unwrap_or(&Value::Null))
        .map_err(|e| Stop::Raised("ValueError".to_string(), e.to_string()))
}

fn opt_float(p: &Map<String, Value>, key: &str) -> R<Option<f64>> {
    match p.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => Ok(Some(py_float(v, &format!("params.{key}"))?)),
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
                    "status": c.status.key(),
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
        // Issue #331 (spec 0006): crude oil in integral tanks. The format only; no Python vessel.
        "tank_barge" => (
            "Tank barge, integral tanks",
            Some("tank"),
            CargoClass::LiquidBulk,
        ),
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

/// `_curves`: speed-power for the hull alone, with the resistance results they came from (issue
/// #353: the powering reads them).
fn curves(
    p: &Map<String, Value>,
    l: f64,
    b: f64,
    t: f64,
    (h, extra_wetted_m2): (HullAt, f64),
    disp: f64,
    rho: f64,
) -> R<(Value, Option<resistance::ResistanceResults>)> {
    let Some(rr) = resistance_of(p, l, b, t, (h, extra_wetted_m2), disp, rho)? else {
        return Ok((json!([]), None));
    };
    let list = Value::Array(
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
    );
    Ok((list, Some(rr)))
}

/// The resistance of the hull in each environment (deep water, and the channel when one is given);
/// `None` with no draught.
fn resistance_of(
    p: &Map<String, Value>,
    l: f64,
    b: f64,
    t: f64,
    (h, extra_wetted_m2): (HullAt, f64),
    disp: f64,
    rho: f64,
) -> R<Option<resistance::ResistanceResults>> {
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
    // Issue #448: a draught so small that nothing is immersed has no waterline either.
    if t <= 0.0 || h.lwl <= 0.0 {
        return Ok(None);
    }
    // Mutation survivors (#64), deleting one of these fields below: equivalent, each for its own
    // reason. `loa` and `displacement_tonnes` are never read by the resistance code (it uses `lwl`,
    // set to the same length, and works out the displacement itself). `speed_step_knots` is 0.5,
    // which is also the default. `engine_power_kw` (1e6 here, 2400 by default) only sets whether a
    // speed is "power available", which feeds the service speed; `simulate` lists neither, and the
    // curve is not cut by it.
    let inputs = ResistanceInputs {
        loa: l,
        // Issue #448: the hull's waterline length, Cb and midship section.
        lwl: h.lwl,
        beam: b,
        draft: t,
        block_coefficient: h.cb,
        // Issue #425: Am = B x T x Cm where no hull section is given; issue #448: a Design gives
        // the hull's own midship area below, so it is the sections'.
        midship_coefficient: get_or(
            p,
            "midship_coefficient",
            ovcd_core::confined_water::DEFAULT_MIDSHIP_COEFFICIENT,
        )?,
        displacement_tonnes: disp,
        midship_area_m2: Some(h.midship_m2),
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
    resistance::compute(&inputs)
        .map(Some)
        .map_err(|e| Stop::Raised(e.type_name().to_string(), e.message().to_string()))
}

/// Issue #420: the resistance method's checks at each service speed (squat, waves, the fits, the
/// Schijf limit), named as the run lists checks: the name read with spaces, the environment in
/// brackets.
fn method_checks(
    res: Option<&resistance::ResistanceResults>,
    out: &mut Vec<Value>,
    ids: &mut TextIds,
) {
    let Some(res) = res else {
        return;
    };
    let kept: Vec<Check> = res
        .checks
        .iter()
        // The run gives the resistance model a stand-in installed power (see `resistance_of`), so
        // this check would test nothing.
        .filter(|c| !c.name.starts_with("power_within_installed["))
        .map(|c| Check {
            name: c.name.replace('_', " "),
            ..c.clone()
        })
        .collect();
    ids.checks(&kept);
    if let Value::Array(items) = checks_json(&kept) {
        out.extend(items);
    }
}

/// Issue #420: a channel with a depth but no bottom width is not a resistance environment (see
/// `resistance_of`), so its squat, Schijf limit and blockage are not checked. Say so.
fn channel_width_note(p: &Map<String, Value>) -> R<Option<Text>> {
    let ch = channel(p)?;
    let has = |k: &str| ch.get(k).is_some_and(truthy);
    Ok((has("depth_m") && !has("bottom_width_m")).then(|| {
        text!(
            "warn-channel-no-bottom-width",
            "The channel has a depth but no bottom width, so squat, the Schijf limiting speed and the blockage are not checked and the speed curves are for deep water only. The draught limit still uses the depth. Give the channel bottom width to check them."
        )
    }))
}

/// The capacity checks as the simulator lists them, the name read with spaces. The two that test
/// the full tank volume (`FULL_VOLUME_CHECKS`) gate nothing, so they are listed as warnings: shown,
/// as every check is, but not a failure (issue #418).
fn capacity_checks(checks: &[Check], out: &mut Vec<Value>) {
    let kept: Vec<Check> = checks
        .iter()
        .map(|c| Check {
            name: c.name.replace('_', " "),
            severity: if ga::FULL_VOLUME_CHECKS.contains(&c.name.as_str()) {
                "warning".to_string()
            } else {
                c.severity.clone()
            },
            ..c.clone()
        })
        .collect();
    if let Value::Array(items) = checks_json(&kept) {
        out.extend(items);
    }
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
        ("hold_count", dry.map_or(Value::Null, |i| json!(i.n_holds))),
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
            "weather_shield_thickness_mm",
            basis.map_or(Value::Null, |b| num(b.weather_shield_thickness)),
        ),
        (
            "weather_shield_clearance_m",
            basis.map_or(Value::Null, |b| num(b.weather_shield_clearance)),
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
        // Issue #354: a fixed wheelhouse does not lift.
        wheelhouse_max_lift_m: if p.get("wheelhouse_type").and_then(Value::as_str) == Some("fixed")
        {
            0.0
        } else {
            get_or(p, "wheelhouse_max_lift_m", d.wheelhouse_max_lift_m)?
        },
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
        // Issue #577: `null` is the IGC 4.23.1.2 minimum of each tank, worked out by the core.
        design_pressure_mpa: if design_pressure_by_rule(p) {
            b::DEFAULT_TANK_DESIGN_PRESSURE_MPA
        } else {
            get_or(
                p,
                "tank_design_pressure_mpa",
                b::DEFAULT_TANK_DESIGN_PRESSURE_MPA,
            )?
        },
        design_pressure_igc_minimum: design_pressure_by_rule(p),
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

/// Whether the design pressure is asked for as the IGC 4.23.1.2 minimum of each tank (issue #577):
/// `tank_design_pressure_mpa` is `null`.
fn design_pressure_by_rule(p: &Map<String, Value>) -> bool {
    p.get("tank_design_pressure_mpa")
        .is_some_and(Value::is_null)
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
        if design_pressure_by_rule(p) {
            return bad(
                "`params.tank_design_pressure_mpa` must be a number for a fully pressurised LPG \
                 Cargo: the MARVS sets the loading temperature (IGC 15.1.3.1)"
                    .to_string(),
            );
        }
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
/// The longitudinal and transverse gaps between tanks. The core lays the tanks out with the defaults
/// of `BargeInputs` (it has no input for them), so this is where they are read for the results.
fn core_tank_gaps() -> (f64, f64) {
    let base = ovcd_core::models::BargeInputs::new(1.0, 1.0, 1.0, 1.0, 1.0);
    (base.tank_gap_longitudinal, base.tank_gap_transverse)
}

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
    // Issue #406: the tank lengths and the two gaps the core laid the tanks out with, so the drawing
    // draws from them and holds no stand-in of its own.
    let (gap_long, gap_across) = core_tank_gaps();
    // The design liquid pressure the wall took (issue #374): Pgd = a x Z x rho / 1.02e5, with Z the
    // steel outer diameter, and Peq = Po + Pgd.
    let liquid = ovcd_core::barge_capacity::liquid_pressure_basis(
        inp,
        r.bow_void_length_m.unwrap_or(0.0),
        r.stern_void_length_m.unwrap_or(0.0),
        gap_long,
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
    // The pressure in force: the typed one, or the IGC minimum the core worked out for the tank.
    let design_pressure = r
        .gas_basis
        .as_ref()
        .map_or(design.design_pressure_mpa, |b| b.tanks.design_pressure);
    Value::Object(
        [
            ("lpg_carriage_condition", text("lpg_carriage_condition")),
            ("design_pressure_mpa", num(design_pressure)),
            ("liquid_pressure", text("tank_liquid_pressure")),
            ("liquid_pressure_accel", opt_num(liquid.0)),
            ("liquid_pressure_density_kg_m3", num(liquid.1)),
            ("liquid_pressure_mpa", num(pgd)),
            ("internal_pressure_mpa", num(design_pressure + pgd)),
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
            (
                "overall_tank_length_m",
                opt_num(r.gas_basis.as_ref().map(|b| b.tanks.overall_tank_length)),
            ),
            (
                "cylinder_length_m",
                opt_num(r.gas_basis.as_ref().map(|b| b.tanks.cylinder_length)),
            ),
            ("tank_gap_longitudinal_m", num(gap_long)),
            ("tank_gap_transverse_m", num(gap_across)),
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
    // Issue #448: the hull as its Sections Table. Its Cb at the design draught is the one the IGC
    // 4.28.2 vertical acceleration of a gas tank reads.
    let hull = option_hull(p, (l, b, d), stern)?;
    let cb = hull_at(&hull, t_design, rho)?.cb;
    if vessel_id == "gas_barge" {
        return simulate_gas(
            p,
            label,
            cargo,
            cargo_key,
            (l, b, d),
            (&hull, cb, rho),
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
    // Issue #448: the same bow as the Option's hull, so the capacity works on that hull.
    let (rake_length, rake_rise) = bow_rake_of(p, d)?;
    inp.bow_rake_length_m = Some(rake_length);
    inp.bow_rake_rise_m = Some(rake_rise);
    inp.hull_shape = hull_shape_of(p)?;
    inp.downflooding_angle_deg = opt_float(p, "downflooding_angle_deg")?;
    inp.downflooding_openings = openings_of(p)?;
    inp.aft_design = aft_design_of(p)?;
    inp.stern_block_mass = stern_mass_of(p)?;
    inp.stern_geometry = stern.cloned();
    inp.machinery = machinery_of(p, stern, (l, b, d), t_design, self_propelled)?;
    size_installed_power(
        p,
        &mut inp.machinery,
        &mut inp.aft_design,
        stern,
        (l, b, d),
        t_design,
        (&hull, rho),
    )?;
    inp.sightline_limit_m = sightline_limit_of(p, self_propelled);
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
    if phase == CargoClass::LiquidBulk {
        inp.liquid = Some(crate::liquid::layout_of(p).or_else(bad)?);
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
    // Issue #448: the displacement at the draught cap, from the sections (level).
    let disp_max = hull.volume_at(py_min(t_max, d)) * rho;
    let cargo_volume_bound = r.cargo_capacity_t;
    // Issue #426: with the IS Code conditions the full load carries the stores and fuel, so the
    // weight-bound cargo is that much less. The fuel is worked out at the full-load draught
    // without them.
    let consumables_t = if is_code(p) {
        let disp0 = ls + py_min(cargo_volume_bound, py_max(0.0, disp_max - ls));
        let t0 = level_draught(&hull, disp0, rho)?;
        let extra = stern.map_or(0.0, |g| g.extra_wetted_surface(t0));
        consumables_full_t(
            p,
            self_propelled,
            &curves(p, l, b, t0, (hull_at(&hull, t0, rho)?, extra), disp0, rho)?.0,
        )?
    } else {
        0.0
    };
    let cargo_weight_bound = if is_code(p) {
        py_max(0.0, disp_max - ls - consumables_t)
    } else {
        py_max(0.0, disp_max - ls)
    };
    let cargo_t = py_min(cargo_volume_bound, cargo_weight_bound);
    let mut ids = TextIds::default();
    let governs = if cargo_volume_bound <= cargo_weight_bound {
        Text::plain(if phase == CargoClass::LiquidBulk {
            "tank volume"
        } else {
            "hold volume"
        })
    } else {
        governs_draught(&lim_name)
    };
    ids.text(&governs);
    let disp = if is_code(p) {
        ls + cargo_t + consumables_t
    } else {
        ls + cargo_t
    };
    let deadweight = disp - ls;
    measured.deadweight_t = Some(deadweight);
    // Issue #337: the liquid bulk layout, for the liquid Measures.
    measured.liquid = r.liquid.clone();
    // Issue #454: what the Vessel Model is assembled from.
    measured.vessel = Some(ovcd_core::vessel_model::VesselModelInputs {
        capacity: inp.clone(),
        result: r.clone(),
        hull: hull.table.clone(),
    });
    // Issue #448: Cb and the hydrostatics at the laden level draught; the reported draught is the
    // one at mid-length of the free-trim float when the hull has LCG data (a dry bulk hull with
    // its stern block mass), with the deadweight at the middle of the hold zone.
    let t_level = level_draught(&hull, disp, rho)?;
    let laden = hull_at(&hull, t_level, rho)?;
    let (hydrostatics, hydrostatic_table) = hydrostatics_json(&hull, t_level, rho)?;
    let t = if phase == CargoClass::DryBulk {
        barge_capacity::bulk_float_draught_at(&inp, &r, deadweight, &hull)
            .map_err(|e| Stop::Raised(e.kind.to_string(), e.message))?
            .unwrap_or(t_level)
    } else {
        t_level
    };
    let h_top = r.air_draft_top_above_keel_m;

    let mut checks = vec![json!({
        "name": "Cargo > 0",
        "ok": cargo_t > 0.0,
        "message": if cargo_t <= 0.0 { "Lightship alone exceeds the allowed draught." } else { "OK" },
        "severity": "error",
    })];
    // Issue #415: the hold GM at the cargo and the draught the Design reports, in every dry bulk
    // Option, in place of the capacity result's GM at the volume-bound load (which it gives only
    // when that load floats).
    let mut capacity: Vec<Check> = r
        .checks
        .iter()
        .filter(|c| c.name != "gm_positive")
        .cloned()
        .collect();
    let mut warnings = r.warnings.clone();
    // `laden.lwl` is 0 when nothing is immersed, even at a draught of a few times 1e-324 m.
    if phase == CargoClass::DryBulk && t_level > 0.0 && laden.lwl > 0.0 && t_level <= d {
        // KM_T of the upright cut that displaces the load.
        let g = barge_capacity::bulk_gm_at(&inp, &r, cargo_t, t_level, &hull)
            .map_err(|e| Stop::Raised(e.kind.to_string(), e.message))?;
        capacity.push(Check::with_severity(
            "intact_gm_at_reported_cargo",
            g >= 0.15,
            text!(
                "check-intact-gm-reported-cargo",
                "intact GM {gm} m >= 0.15 m at the reported cargo {cargo} t, draught {draught} m (first-order; cargo KG of the full hold; KG_light @0.5D)",
                gm = Arg::fixed(g, 2),
                cargo = Arg::fixed(cargo_t, 0),
                draught = Arg::fixed(t_level, 2),
            ),
            "warning",
        ));
        let note = barge_capacity::bulk_gm_note();
        if !warnings.contains(&note) {
            warnings.push(note);
        }
    }
    capacity_checks(&capacity, &mut checks);
    ids.checks(&capacity);
    ids.texts(&warnings);
    let (curve_list, resistance) = curves(
        p,
        l,
        b,
        t,
        (laden, stern.map_or(0.0, |g| g.extra_wetted_surface(t))),
        disp,
        rho,
    )?;
    method_checks(resistance.as_ref(), &mut checks, &mut ids);
    if let Some(note) = channel_width_note(p)? {
        ids.text(&note);
        warnings.push(note);
    }
    let powering_block = powering_json(p, inp.machinery.as_ref(), stern, rho, resistance.as_ref())?;
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
    // Issue #426: the IS Code conditions, with the ballast tanks of the hold arrangement.
    let mut is_code_stability = Value::Null;
    let mut loading_conditions = Value::Null;
    // Issue #426: the ballast tanks of a dry bulk hull, for the drawing (not hashed).
    let mut ballast_drawing: Option<(&'static str, Value)> = None;
    // Issue #332 (spec 0006): the liquid bulk conditions are the IS Code ones with the cargo tanks
    // of the layout, and the worst slack-tank condition besides.
    if is_code(p) && phase == CargoClass::LiquidBulk {
        // A layout that does not fit has no Spaces (the `feasible geometry` check says why): there are
        // no conditions to assess.
        if let Some((lightship, liquid)) = ovcd_core::loading_conditions::liquid_model(&r) {
            let cargo_at = |m: f64| liquid.at(m);
            let tanks = clipped_ballast_tanks(&inp, &r, &hull);
            let slack = liquid.slack_masses_t();
            let conds = is_code_conditions(
                p,
                &IsCodeHull {
                    dims: (l, b, d),
                    hull: &hull,
                    rho,
                    criteria_set: gs::DEFAULT_CRITERIA_SET,
                    lightship: &lightship,
                    full_cargo_t: cargo_t,
                    cargo_at: &cargo_at,
                    ballast_tanks: &tanks,
                    consumables_t,
                    air_draft_top_m: h_top,
                    self_propelled,
                    slack_candidates_t: &slack,
                },
            )?;
            loading_condition_checks(&conds, "warning", false, &mut checks, &mut ids);
            is_code_stability = is_code_stability_json(&conds, true, &mut ids);
            loading_conditions = loading_conditions_json(&conds, &tanks, consumables_t, rho, true);
            // Issue #341: the assumed damages of ADN 9.3.x.15 on the laden departure condition.
            if let Some(liquid) = r.liquid.as_ref() {
                let marpol = rules.sets.iter().any(|s| s.id == "marpol-i");
                (measured.damage, measured.marpol_damage) =
                    crate::liquid::damage_verdicts(&hull, liquid, cargo_key, rho, &conds, marpol);
            }
            // Issue #341: the mean oil outflow of MARPOL Annex I regulation 23, at the draught of the full load.
            if let Some(liquid) = r.liquid.as_ref()
                && rules.sets.iter().any(|s| s.id == "marpol-i")
            {
                measured.oil_outflow = ovcd_core::liquid_bulk::oil_outflow::oil_outflow(
                    &liquid.spaces,
                    &ovcd_core::liquid_bulk::oil_outflow::OilOutflowInputs {
                        length_m: l,
                        beam_m: b,
                        depth_m: d,
                        draught_m: t_level,
                        deadweight_t: deadweight,
                        overpressure_kpa: 5.0,
                    },
                );
            }
            // The liquid conditions stay out of the Rule Trace until the liquid Rule Sets exist: the
            // IS Code Rule Set would block a low-freeboard tank barge on seagoing criteria.
        }
    } else if is_code(p) {
        let (lightship, cargo_item) = ovcd_core::loading_conditions::dry_model(&inp, &r);
        let cargo_at = |m: f64| {
            Ok(ovcd_core::loading_conditions::CargoAt {
                item: ovcd_core::loading_conditions::Item {
                    mass_t: m,
                    ..cargo_item
                },
                // The part of the volume-bound load: a draught-bound load fills the hold less.
                fill_fraction: if r.cargo_capacity_t > 0.0 {
                    py_min(1.0, m / r.cargo_capacity_t)
                } else {
                    0.0
                },
                free_surface_moment_tm: 0.0,
            })
        };
        let raw = ovcd_core::compartments::ballast_tanks(&inp, &r);
        ballast_drawing = Some(crate::liquid::ballast_drawing_value(&raw));
        let tanks: Vec<_> = raw
            .iter()
            .map(|t| ovcd_core::compartments::clip(t, &hull.table))
            .collect();
        let conds = is_code_conditions(
            p,
            &IsCodeHull {
                dims: (l, b, d),
                hull: &hull,
                rho,
                criteria_set: gs::DEFAULT_CRITERIA_SET,
                lightship: &lightship,
                full_cargo_t: cargo_t,
                cargo_at: &cargo_at,
                ballast_tanks: &tanks,
                consumables_t,
                air_draft_top_m: h_top,
                self_propelled,
                slack_candidates_t: &[],
            },
        )?;
        loading_condition_checks(&conds, "warning", false, &mut checks, &mut ids);
        is_code_stability = is_code_stability_json(&conds, false, &mut ids);
        loading_conditions = loading_conditions_json(&conds, &tanks, consumables_t, rho, false);
        // The dry bulk conditions stay out of the Rule Trace: the IS Code Rule Set would block a
        // low-freeboard hold on seagoing criteria. That waits for a dry bulk Rule Set (triage
        // item 2) and the `fit` seam (#277).
    } else if phase == CargoClass::LiquidBulk {
        // The legacy loading conditions are the gas ones: liquid bulk has the IS Code ones only.
        let note = Text::fixed(
            "warn-liquid-legacy-conditions",
            "Stability is computed for liquid bulk cargo with the IS Code loading conditions only.",
        );
        ids.text(&note);
        warnings.push(note);
    }
    let mut dry_parts = structure_json(
        &r,
        inp.separation.as_ref(),
        (phase != CargoClass::LiquidBulk).then_some(&inp),
    );
    dry_parts.extend(crate::liquid::drawing_values(&r));
    dry_parts.extend(ballast_drawing);
    if let Some(g) = stern {
        dry_parts.push(("stern", stern_drawing(g)));
    }
    // Issue #450: the hull's own midship section, for the GA.
    dry_parts.extend(midship_outline(&hull));
    let drawing = drawing_json(dry_parts);
    with_blocks_and_text_ids(
        powering_block,
        sightline_json(&r),
        with_loading_conditions(
            with_stern_mass(
                with_liquid(
                    json!({
                        "vessel": label,
                        "cargo": cargo.name,
                        "phase": cargo.phase.key(),
                        "self_propelled": self_propelled,
                        "arrangement": Value::Null,
                        "dims": { "loa_m": num(l), "beam_m": num(b), "depth_m": num(d), "cb": num(if t > 0.0 { laden.cb } else { cb }) },
                        "hydrostatics": hydrostatics,
                        "hydrostatic_table": hydrostatic_table,
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
                        "warnings": warnings,
                        "curves": curve_list,
                        "cost_masses": Value::Null,
                    }),
                    &r,
                    cargo_key,
                    &measured.damage,
                    &measured.marpol_damage,
                    measured.oil_outflow.as_ref(),
                ),
                (&r.stern_block_items, &r.trim_conditions),
            ),
            is_code_stability,
            loading_conditions,
        ),
        ids,
    )
}

/// Issue #331: the `liquid` block, for a liquid bulk Cargo only, so a dry bulk result keeps its keys.
fn with_liquid(
    mut v: Value,
    r: &BargeCapacityResult,
    cargo_key: &str,
    damage: &[ovcd_core::measured::DamageVerdict],
    marpol_damage: &[ovcd_core::measured::DamageVerdict],
    oil_outflow: Option<&ovcd_core::liquid_bulk::oil_outflow::OilOutflow>,
) -> Value {
    if let (Some(l), Some(o)) = (r.liquid.as_ref(), v.as_object_mut()) {
        o.insert(
            "liquid".to_string(),
            crate::liquid::liquid_json(l, cargo_key),
        );
        // Issue #334: what the cargo tank items of the budget are priced on (unhashed, with the rest of
        // `cost_masses`).
        o.insert("cost_masses".to_string(), crate::cost::liquid_masses(l));
        // Issue #341: what each assumed damage does (unhashed: it is an output of the Rule Trace's input).
        if !damage.is_empty() {
            o.insert("damage".to_string(), crate::liquid::damage_json(damage));
        }
        if let Some(oo) = oil_outflow {
            o.insert(
                "oil_outflow".to_string(),
                crate::liquid::oil_outflow_json(oo),
            );
        }
        if !marpol_damage.is_empty() {
            o.insert(
                "marpol_damage".to_string(),
                crate::liquid::damage_json(marpol_damage),
            );
        }
    }
    v
}

/// Issue #426: the IS Code `stability` and `loading_conditions` blocks, only when they were asked
/// for, so an old file's results keep their keys and their hash.
fn with_loading_conditions(mut v: Value, stability: Value, loading: Value) -> Value {
    if !loading.is_null() {
        v["stability"] = stability;
        v["stability_marks"] = json!({ "criteria_angles_deg": gs::CRITERIA_ANGLES_DEG });
        v["loading_conditions"] = loading;
    }
    v
}

/// "draught — <limit>": the draught governs the cargo, and the limit sets the draught.
fn governs_draught(limit: &str) -> Text {
    text!(
        "name-governs-draught",
        "draught \u{2014} {limit}",
        limit = limit
    )
}

/// The result with its `text_ids` (ADR 0015): a reading aid, outside the results hash.
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

/// Issue #353: the Propulsor and the Power Source of a machinery, for the powering. The wake is
/// spec 0007's: 0.30 under a tunnel, 0.20 at a transom or pram stern (estimates).
fn powering_inputs(
    p: &Map<String, Value>,
    m: &ovcd_core::machinery::MachineryInputs,
    stern: Option<&ovcd_core::stern_form::SternGeometry>,
    rho: f64,
    installed_power_kw: Option<f64>,
) -> R<ovcd_core::powering::PoweringInputs> {
    use ovcd_core::propulsor::{PropulsorInputs, WAKE_OPEN_STERN, WAKE_TUNNEL};
    let wake = match stern.map(|g| g.form) {
        Some(ovcd_core::stern_form::SternForm::Tunnel) => WAKE_TUNNEL,
        _ => WAKE_OPEN_STERN,
    };
    Ok(ovcd_core::powering::PoweringInputs {
        propulsor: PropulsorInputs {
            propulsor_type: m.propulsor,
            count: m.propulsor_count,
            diameter_m: m.propeller_diameter_m,
            wake,
            kappa: get_or(p, "propulsor_efficiency_factor", 1.0)?,
            kappa_b: get_or(p, "bollard_pull_factor", 1.0)?,
            rho_kg_m3: rho * 1000.0,
        },
        power_source: m.power_source,
        design_speed_kn: get_or(p, "design_speed_kn", 9.7)?,
        margin: get_or(p, "installed_power_margin", 1.2)?,
        installed_power_kw,
        trip_length_km: truthy_float(p, "trip_length_km")?,
    })
}

/// The resistance curves of the powering (issue #353), from the resistance results.
fn powering_curves(
    rr: &resistance::ResistanceResults,
) -> Vec<ovcd_core::powering::EnvironmentCurve> {
    rr.environments
        .iter()
        .map(|e| ovcd_core::powering::EnvironmentCurve {
            name: e.name.clone(),
            points: e
                .speed_curve
                .iter()
                .map(|s| ovcd_core::powering::CurvePoint {
                    speed_kn: s.speed_knots,
                    speed_through_water_kn: s.speed_through_water_knots,
                    resistance_kn: s.resistance_total_kn,
                    squat_limited: s.squat_limited,
                })
                .collect(),
        })
        .collect()
}

/// Issue #353: size the Installed Power of a machinery that has none typed, at the design draught,
/// and give it to the machinery (its engines and their room). The laden draught is not known before
/// the machinery's mass is, so the design draught (the deepest the hull loads to) stands for it.
/// The battery follows its energy, typed or from the trip: a battery-electric block carries as many
/// units as the energy needs, a hybrid's battery room holds it.
fn size_installed_power(
    p: &Map<String, Value>,
    machinery: &mut Option<ovcd_core::machinery::MachineryInputs>,
    aft_design: &mut ovcd_core::aft_arrangement::AftDesign,
    stern: Option<&ovcd_core::stern_form::SternGeometry>,
    (l, b, _d): (f64, f64, f64),
    t_design: f64,
    (hull, rho): (&Hull, f64),
) -> R<()> {
    use ovcd_core::machinery::PowerSource;
    let Some(m) = machinery else {
        return Ok(());
    };
    let extra = stern.map_or(0.0, |g| g.extra_wetted_surface(t_design));
    // Issue #448: the displacement at the design draught, from the sections.
    let disp = hull.volume_at(t_design) * rho;
    let h = hull_at(hull, t_design, rho)?;
    let Some(rr) = resistance_of(p, l, b, t_design, (h, extra), disp, rho)? else {
        return Ok(());
    };
    let pw = ovcd_core::powering::compute(
        &powering_inputs(p, m, stern, rho, m.installed_power_kw)?,
        &powering_curves(&rr),
    );
    if m.installed_power_kw.is_none() && pw.installed_power_kw > 0.0 {
        m.installed_power_kw = Some(pw.installed_power_kw);
    }
    if let Some(e) = m.battery_energy_kwh.or(pw.battery_energy_kwh) {
        // The energy in force, sized at the design draught like the power.
        m.battery_energy_kwh = Some(e);
        if m.power_source == PowerSource::BatteryElectric {
            let unit = get_or(p, "battery_unit_energy_kwh", 5000.0)?;
            if unit > 0.0 {
                aft_design.n_battery_units = Some(libm::ceil(e / unit) as i64);
            }
        }
    }
    Ok(())
}

/// The energy of a stern block's battery units (kWh): their count x the energy of one.
fn battery_units_kwh(p: &Map<String, Value>, r: &BargeCapacityResult) -> f64 {
    let n = r.aft_arrangement.as_ref().map_or(0, |a| {
        a.container_labels
            .iter()
            .filter(|l| l.starts_with("BATTERY"))
            .count()
    });
    n as f64 * get_or(p, "battery_unit_energy_kwh", 5000.0).unwrap_or(5000.0)
}

/// Issue #353: the `powering` block of a machinery's results, at the laden draught's curves; `None`
/// with no machinery (today's curves stand alone).
fn powering_json(
    p: &Map<String, Value>,
    machinery: Option<&ovcd_core::machinery::MachineryInputs>,
    stern: Option<&ovcd_core::stern_form::SternGeometry>,
    rho: f64,
    rr: Option<&resistance::ResistanceResults>,
) -> R<Option<Value>> {
    let (Some(m), Some(rr)) = (machinery, rr) else {
        return Ok(None);
    };
    let sized = truthy_float(p, "installed_power_kw")?.is_none();
    let pw = ovcd_core::powering::compute(
        &powering_inputs(p, m, stern, rho, m.installed_power_kw)?,
        &powering_curves(rr),
    );
    Ok(Some(json!({
        "installed_power_kw": num(pw.installed_power_kw),
        "installed_power_sized": sized,
        "design_speed_kn": num(get_or(p, "design_speed_kn", 9.7)?),
        "margin": num(get_or(p, "installed_power_margin", 1.2)?),
        "bollard_pull_t": num(pw.bollard_pull_t),
        // The battery in force: typed, else from the trip (none without a battery or a route).
        "battery_energy_kwh": m.battery_energy_kwh.or(pw.battery_energy_kwh).map_or(Value::Null, num),
        "environments": pw.environments.iter().map(|e| json!({
            "name": e.name,
            "service_speed_kn": num(e.service_speed_kn),
            "power_at_service_kw": num(e.needed_at_service_kw),
            "fuel_kg_per_h": num(e.fuel_kg_per_h),
            "energy_per_trip_kwh": e.energy_per_trip_kwh.map_or(Value::Null, num),
        })).collect::<Vec<_>>(),
    })))
}

/// Issue #354: the Blind Distance limit of a self-propelled hull whose sightline is checked as a
/// distance (ES-TRIN 7.02(2), 250 m until the Rule Sets of #355); `None` for the margin method.
fn sightline_limit_of(p: &Map<String, Value>, self_propelled: bool) -> Option<f64> {
    let method = p.get("sightline_method").and_then(Value::as_str);
    (self_propelled && method == Some("blind_distance"))
        .then_some(ovcd_core::wheelhouse::ES_TRIN_BLIND_DISTANCE_M)
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

/// The bow rake of the Option (issue #448, as `barge_capacity` takes it): `bow_rake_length_m`, else
/// the ratio x depth; it rises to `bow_rake_rise_m`, else the depth.
fn bow_rake_of(p: &Map<String, Value>, d: f64) -> R<(f64, f64)> {
    let ratio = get_or(
        p,
        "bow_rake_length_to_depth_ratio",
        ovcd_core::barge_capacity::DEFAULT_BOW_RAKE_TO_DEPTH_RATIO,
    )?;
    let length = opt_float(p, "bow_rake_length_m")?.unwrap_or(ratio * d);
    let rise = opt_float(p, "bow_rake_rise_m")?.unwrap_or(d);
    Ok((length, rise))
}

/// The drawing key `midship_outline` (issue #450): the starboard half-section of the hull at
/// mid-length, from the keel to the deck edge, as [y, z] pairs. Only for a hull whose section is
/// not the rectangle (a bilge radius, a V-bottom or a chine); the GA draws the rectangle otherwise.
fn midship_outline(hull: &Hull) -> Option<(&'static str, Value)> {
    let shape = &hull.spec.shape;
    if shape.bilge_radius_m <= 0.0 && shape.bottom == ovcd_core::hull::hull_form::Bottom::Flat {
        return None;
    }
    let (section, edge) = hull.table.section_at(hull.spec.loa / 2.0);
    let half = hull.spec.beam / 2.0;
    let end = section
        .points
        .iter()
        .position(|&(y, z)| y == half && z == edge)
        .unwrap_or(section.points.len() - 1);
    let points: Vec<Value> = section.points[..=end]
        .iter()
        .map(|&(y, z)| json!([num(y), num(z)]))
        .collect();
    Some(("midship_outline", Value::Array(points)))
}

/// The bilge radius, sheer and camber of the Option's hull (issue #449), and its bottom (issue
/// #450): a V-bottom or chine bottom for those Hull Forms, else flat. A key left out is 0, the box
/// of a file saved before.
fn hull_shape_of(p: &Map<String, Value>) -> R<ovcd_core::hull::hull_form::HullShape> {
    use ovcd_core::hull::hull_form::Bottom;
    let bottom = match p.get("hull_form").and_then(Value::as_str) {
        Some("v_bottom") => Bottom::VBottom {
            deadrise_deg: need(p, "deadrise_deg")?,
            keel_half_width_m: need(p, "keel_half_width_m")?,
        },
        Some("chine") => Bottom::Chine {
            deadrise_deg: need(p, "deadrise_deg")?,
            keel_half_width_m: need(p, "keel_half_width_m")?,
            chine_half_breadth_m: opt_float(p, "chine_half_breadth_m")?,
            chine_height_m: opt_float(p, "chine_height_m")?,
        },
        _ => Bottom::Flat,
    };
    Ok(ovcd_core::hull::hull_form::HullShape {
        bilge_radius_m: get_or(p, "bilge_radius_m", 0.0)?,
        sheer_aft_m: get_or(p, "sheer_aft_m", 0.0)?,
        sheer_fwd_m: get_or(p, "sheer_fwd_m", 0.0)?,
        camber_m: get_or(p, "camber_m", 0.0)?,
        bottom,
        offsets: offsets_of(p)?,
        shipd: shipd_of(p)?,
        notch: notch_of(p)?,
    })
}

/// The Notch of an ATB barge (issue #362): typed, or half the tug's length deep and the tug's beam
/// and 0.30 m wide, the tug itself typed or 0.28 of the barge's length and 0.53 of its beam. `None`
/// for any other Formation.
fn notch_of(p: &Map<String, Value>) -> R<Option<ovcd_core::stern_form::Notch>> {
    if !p.get("atb").is_some_and(truthy) {
        return Ok(None);
    }
    let tug = ovcd_core::atb::tug_from_barge(
        need(p, "loa_m")?,
        need(p, "beam_m")?,
        ovcd_core::atb::TugTyped {
            length_m: opt_float(p, "tug_length_m")?,
            beam_m: opt_float(p, "tug_beam_m")?,
            draught_m: opt_float(p, "tug_draught_m")?,
        },
    );
    Ok(Some(ovcd_core::atb::notch_for_tug(
        &tug,
        opt_float(p, "notch_depth_m")?,
        opt_float(p, "notch_width_m")?,
    )))
}

/// The hull of the params as its Sections Table, as the run builds it (issue #451): the Hull
/// Form, its shape, the raked bow and the Stern Form at the design draught.
pub(crate) fn hull_table_of(p: &Map<String, Value>) -> R<ovcd_core::sections::SectionsTable> {
    let (l, b, d) = (need(p, "loa_m")?, need(p, "beam_m")?, need(p, "depth_m")?);
    let self_propelled = p.get("self_propelled").is_some_and(truthy);
    let stern = stern_of(p, (l, b, d), design_draught(p, d)?, self_propelled)?;
    Ok(option_hull(p, (l, b, d), stern.as_ref())?.table)
}

/// The typed offsets table of an `offsets` Option (issue #451), as its Sections Table; `None` for
/// any other Hull Form.
fn offsets_of(p: &Map<String, Value>) -> R<Option<ovcd_core::sections::SectionsTable>> {
    if p.get("hull_form").and_then(Value::as_str) != Some("offsets") {
        return Ok(None);
    }
    let rows = crate::offsets::rows_of(p.get("hull_offsets").unwrap_or(&Value::Null))
        .map_err(|e| Stop::Raised("ValueError".to_string(), e.to_string()))?;
    let table = ovcd_core::offsets::from_rows(&rows)
        .map_err(|e| Stop::Raised("ValueError".to_string(), format!("{e}")))?;
    Ok(Some(table))
}

/// The ShipD parameters of a `shipd` Option (issue #452); `None` for any other Hull Form.
fn shipd_of(p: &Map<String, Value>) -> R<Option<ovcd_core::shipd::ShipdParameters>> {
    if p.get("hull_form").and_then(Value::as_str) != Some("shipd") {
        return Ok(None);
    }
    let (l, b, d) = (need(p, "loa_m")?, need(p, "beam_m")?, need(p, "depth_m")?);
    crate::shipd::parameters_of(p.get("shipd_parameters").unwrap_or(&Value::Null), (l, b, d))
        .map(Some)
        .map_err(|e| Stop::Raised("ValueError".to_string(), e.to_string()))
}

/// The hull of the Option as its Sections Table (issue #448): the raked bow and the Stern Form.
fn option_hull(
    p: &Map<String, Value>,
    (l, b, d): (f64, f64, f64),
    stern: Option<&ovcd_core::stern_form::SternGeometry>,
) -> R<Hull> {
    let (length, rise) = bow_rake_of(p, d)?;
    Hull::build(&HullSpec {
        loa: l,
        beam: b,
        depth: d,
        bow_rake_length_m: length,
        bow_rake_rise_m: rise,
        stern: stern.cloned(),
        shape: hull_shape_of(p)?,
        downflooding_deg: opt_float(p, "downflooding_angle_deg")?,
        openings: openings_of(p)?,
    })
    .map_err(|e| Stop::Raised("ValueError".to_string(), format!("{e}")))
}

/// What the resistance and squat read from the hull at a draught (issue #448): Cb = ∇ / (L_WL
/// B_WL T), the waterline length and the midship section area.
#[derive(Clone, Copy)]
struct HullAt {
    cb: f64,
    lwl: f64,
    midship_m2: f64,
}

fn hull_at(hull: &Hull, t: f64, rho: f64) -> R<HullAt> {
    let none = HullAt {
        cb: 0.0,
        lwl: 0.0,
        midship_m2: 0.0,
    };
    if t <= 0.0 {
        // No draught: nothing to read (the resistance has no curve then).
        return Ok(none);
    }
    let u = match hull.upright(t, rho) {
        Ok(u) => u,
        // A draught so small that nothing is immersed (a few times 1e-324 m): as no draught.
        Err(ovcd_core::sections::SectionsError::NotAfloat) => return Ok(none),
        Err(e) => return Err(Stop::Raised("ValueError".to_string(), e.to_string())),
    };
    Ok(HullAt {
        cb: u.cb,
        lwl: u.immersed.l_wl_m,
        midship_m2: u.immersed.midship_area_m2,
    })
}

/// The upright hydrostatics of one draught, as the results give them (issue #448).
fn upright_json(u: &ovcd_core::sections::Upright) -> Value {
    let im = &u.immersed;
    json!({
        "draught_m": num(u.draught_m),
        "volume_m3": num(im.volume_m3),
        "displacement_t": num(u.displacement_t),
        "kb_m": num(im.kb_m),
        "lcb_m": num(im.lcb_m),
        "bm_t_m": num(u.bm_t_m),
        "bm_l_m": num(u.bm_l_m),
        "km_t_m": num(u.km_t_m),
        "km_l_m": num(u.km_l_m),
        "waterplane_area_m2": num(im.waterplane_area_m2),
        "lcf_m": num(im.lcf_m),
        "l_wl_m": num(im.l_wl_m),
        "b_wl_m": num(im.b_wl_m),
        "midship_area_m2": num(im.midship_area_m2),
        "wetted_surface_m2": num(im.wetted_surface_m2),
        "tpc_t": num(u.tpc_t),
        "mct_1cm_tm": num(u.mct_1cm_tm),
        "cb": num(u.cb),
        "cm": num(u.cm),
        "cp": num(u.cp),
        "cwp": num(u.cwp),
    })
}

/// The results' `hydrostatics` (the upright values at the laden draught, level; null with no
/// draught) and `hydrostatic_table` (10 draughts, 0.1 D to D) of a hull (issue #448).
fn hydrostatics_json(hull: &Hull, t: f64, rho: f64) -> R<(Value, Value)> {
    let raised = |e: ovcd_core::sections::SectionsError| {
        Stop::Raised("ValueError".to_string(), e.to_string())
    };
    let at = if t > 0.0 {
        match hull.upright(t, rho) {
            Ok(u) => upright_json(&u),
            // A draught so small that nothing is immersed: as no draught.
            Err(ovcd_core::sections::SectionsError::NotAfloat) => Value::Null,
            Err(e) => return Err(raised(e)),
        }
    } else {
        Value::Null
    };
    let table = hull.hydrostatic_table(rho).map_err(raised)?;
    Ok((at, Value::Array(table.iter().map(upright_json).collect())))
}

/// The level draught at which the hull displaces `mass_t` (issue #448); the depth when the whole
/// hull cannot float it, and 0 with no mass.
fn level_draught(hull: &Hull, mass_t: f64, rho: f64) -> R<f64> {
    if mass_t <= 0.0 {
        return Ok(0.0);
    }
    if rho == 0.0 {
        return Err(Stop::Raised(
            "ZeroDivisionError".to_string(),
            "float division by zero".to_string(),
        ));
    }
    if mass_t / rho >= hull.volume_at(hull.spec.depth) {
        return Ok(hull.spec.depth);
    }
    hull.level_draught(mass_t, rho)
        .map_err(|e| Stop::Raised("ValueError".to_string(), e.to_string()))
}

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

/// Issue #354: the sightline as a Blind Distance, in the results only with that method (other
/// results keep their hash).
fn sightline_json(r: &BargeCapacityResult) -> Option<Value> {
    let s = r.sightline.as_ref()?;
    Some(json!({
        "method": "blind_distance",
        "limit_m": num(s.limit_m),
        "blind_distance_light_m": num(s.blind_distance_m),
        "blind_distance_laden_m": r.blind_distance_laden_m.map_or(Value::Null, num),
        "lift_needed_m": num(s.lift_needed_m),
        "lift_used_m": num(s.lift_used_m),
        "eye_raised_m": num(s.eye_raised_m),
        "floor_raised_m": num(s.floor_raised_m),
        "lift_enough": s.enough,
    }))
}

/// The `powering` block (issue #353) joins a machinery's results only, and the `sightline` block
/// (issue #354) a Blind Distance's only, so other results keep their hash; then the Message Ids.
fn with_blocks_and_text_ids(
    powering: Option<Value>,
    sightline: Option<Value>,
    mut result: Value,
    ids: TextIds,
) -> R<Value> {
    if let Some(block) = powering {
        result["powering"] = block;
    }
    if let Some(block) = sightline {
        result["sightline"] = block;
    }
    with_text_ids(result, ids)
}

fn with_text_ids(mut result: Value, ids: TextIds) -> R<Value> {
    result["text_ids"] = ids.finish(&result);
    Ok(result)
}

/// Tank filling limit by ruleset: IGC 15.3 default 98 %, for both IGC rulesets.
fn fill_by_ruleset(_ruleset: &str) -> f64 {
    ovcd_core::tanks::DEFAULT_FILLING_LIMIT
}

#[allow(clippy::too_many_arguments)] // one call site; the run's Rule Sets join the simulate params
fn simulate_gas(
    p: &Map<String, Value>,
    label: &str,
    cargo: &CargoProperties,
    cargo_key: &str,
    (l, b, d0): (f64, f64, f64),
    (hull, cb, rho): (&Hull, f64, f64),
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
    let mut gas_machinery =
        machinery_of(p, stern, (l, b, d0), design_draught(p, d0)?, self_propelled)?;
    let mut gas_aft_design = aft_design_of(p)?;
    size_installed_power(
        p,
        &mut gas_machinery,
        &mut gas_aft_design,
        stern,
        (l, b, d0),
        design_draught(p, d0)?,
        (hull, rho),
    )?;
    // Issue #426: with the IS Code conditions the full load carries the stores and fuel. The fuel
    // is worked out at the design draught, the full-load draught the installed power is sized at,
    // before the search, so the search checks and ranks its candidates with them aboard.
    let is_code_options = if is_code(p) {
        let t_design = design_draught(p, d0)?;
        let extra = stern.map_or(0.0, |g| g.extra_wetted_surface(t_design));
        let disp = hull.volume_at(t_design) * rho;
        Some(ga::IsCodeConditions {
            consumables_t: consumables_full_t(
                p,
                self_propelled,
                &curves(
                    p,
                    l,
                    b,
                    t_design,
                    (hull_at(hull, t_design, rho)?, extra),
                    disp,
                    rho,
                )?
                .0,
            )?,
            may_sail_without_ballast: may_sail_without_ballast(p, self_propelled),
            user_ballast: user_ballast(p)?,
        })
    } else {
        None
    };
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
        // Issue #448: a typed bow rake and rise hold for every candidate depth.
        bow_rake_length_m: opt_float(p, "bow_rake_length_m")?,
        bow_rake_rise_m: opt_float(p, "bow_rake_rise_m")?,
        hull_shape: hull_shape_of(p)?,
        downflooding_angle_deg: opt_float(p, "downflooding_angle_deg")?,
        downflooding_openings: openings_of(p)?,
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
        weather_shield_thickness_mm: get_or(
            p,
            "weather_shield_thickness_mm",
            ovcd_core::barge_capacity::DEFAULT_WEATHER_SHIELD_THICKNESS_MM,
        )?,
        weather_shield_clearance_m: get_or(
            p,
            "weather_shield_clearance_m",
            ovcd_core::barge_capacity::DEFAULT_WEATHER_SHIELD_CLEARANCE_M,
        )?,
        // A key left out keeps the original engine, which weighed neither (issue #412); the
        // Defaults Profile gives a Design its Estimates.
        tank_forming_allowance_pct: get_or(p, "tank_forming_allowance_pct", 0.0)?,
        tank_dome_wall_thickness_mm: get_or(p, "tank_dome_wall_thickness_mm", 0.0)?,
        aft_end_space_m: get_or(
            p,
            "aft_end_space_m",
            ovcd_core::gas_separation::default_end_space(false),
        )?,
        fwd_end_space_m: get_or(
            p,
            "fwd_end_space_m",
            ovcd_core::gas_separation::default_end_space(false),
        )?,
        // Issue #381: a key left out measures the setback from the cargo area; a file saved before
        // keeps `tank_end` through its legacy value.
        opening_setback_from_cargo_area: p
            .get("opening_setback_basis")
            .and_then(Value::as_str)
            .is_none_or(|b| b != "tank_end"),
        tank_wall_thickness_mm: tank_wall_of(p)?,
        tank_design: tank_design_of(p)?,
        water_density_t_per_m3: rho,
        required_freeboard_m: fb,
        air_draft_limit_m: truthy_float(p, "air_draft_limit_m")?,
        aft_design: gas_aft_design,
        stern_block_mass: stern_mass_of(p)?,
        stern_geometry: stern.cloned(),
        rules: rules.sets.clone(),
        rule_inputs: rules.inputs.clone(),
        machinery: gas_machinery,
        sightline_limit_m: sightline_limit_of(p, self_propelled),
        is_code: is_code_options,
        ..FitOptions::default()
    };
    let f = ga::fit(l, b, cargo_key, t_max, &options)
        .map_err(|e| Stop::Raised(e.kind.to_string(), e.message))?;
    let r = &f.engine_result;
    let d = f.depth_m;
    // Issue #448: the hull of the arrangement found, as its sections.
    let fit_hull = match &r.hull_spec {
        Some(spec) => {
            Hull::build(spec).map_err(|e| Stop::Raised("ValueError".to_string(), format!("{e}")))?
        }
        None => hull.clone(),
    };
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
    // Issue #426: the full load carries the stores and fuel; a draught-bound cargo is that much less.
    let consumables_t = options.is_code.as_ref().map_or(0.0, |ic| ic.consumables_t);
    let (cargo_t, disp, t) = if let Some(ic) = &options.is_code {
        let cargo = ga::is_code_cargo(r, f.payload_t, l, b, t_max, &options, ic);
        let disp = r.lightship_t + cargo + consumables_t;
        let t = if cargo > 0.0 {
            level_draught(&fit_hull, disp, rho)?
        } else {
            t
        };
        (cargo, disp, t)
    } else {
        (f.payload_t, disp, t)
    };
    // Issue #448: Cb in the results is the hull's at the laden level draught (no cargo: at the
    // design draught).
    let laden_cb = if t > 0.0 {
        hull_at(&fit_hull, t, rho)?.cb
    } else {
        cb
    };
    let (hydrostatics, hydrostatic_table) = hydrostatics_json(&fit_hull, t, rho)?;
    let deadweight = if is_code(p) {
        cargo_t + consumables_t
    } else {
        f.payload_t
    };
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
        "severity": "error",
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
            "severity": "error",
        }));
    }
    capacity_checks(&r.checks, &mut checks);
    ids.checks(&r.checks);
    ids.texts(&r.warnings);
    // The search's own checks of the answer (the journey, tank length over diameter, freeboard
    // at the payload): every check that can remove a candidate is listed (issue #418).
    let search = ga::answer_search_checks(&f, &options)
        .map_err(|e| Stop::Raised(e.kind.to_string(), e.message))?;
    ids.checks(&search);
    let search: Vec<Check> = search
        .into_iter()
        .map(|c| Check {
            name: c.name.replace('_', " "),
            ..c
        })
        .collect();
    if let Value::Array(items) = checks_json(&search) {
        checks.extend(items);
    }
    let mut stability = Value::Null;
    // What a view marks on the GZ curves. Not a result: it is left out of the results hash.
    let mut stability_marks = Value::Null;
    // The loading conditions, typed, for the Rule Trace.
    let mut conditions = Vec::new();
    let mut is_code_stability = Value::Null;
    let mut loading_conditions = Value::Null;
    if let Some(ic) = options.is_code.as_ref().filter(|_| r.gas_basis.is_some()) {
        // The same conditions the search checked the arrangement in.
        let tanks = clipped_ballast_tanks(&f.engine_inputs, r, &fit_hull);
        let conds = ga::is_code_conditions(r, &f.engine_inputs, cargo_t, &options, ic)
            .map_err(|e| Stop::Raised(e.type_name().to_string(), e.message().to_string()))?;
        let warns = ga::light_angle_warns(&options.rules);
        loading_condition_checks(&conds, "error", warns, &mut checks, &mut ids);
        is_code_stability = is_code_stability_json(&conds, false, &mut ids);
        loading_conditions = loading_conditions_json(&conds, &tanks, consumables_t, rho, false);
        conditions = conds.iter().map(|c| c.condition.clone()).collect();
    } else if f.payload_t > 0.0 && r.gas_basis.is_some() {
        let basis = GasCapacityResult::try_from(r)
            .map_err(|m| Stop::Eval(EvalError::BadInput(m.into())))?;
        let a = gs::assess(&basis, f.payload_t, rho, gs::criteria_set_for(ruleset))
            .map_err(|e| Stop::Raised(e.type_name().to_string(), e.message().to_string()))?;
        for c in &a.conditions {
            let k = ga::gas_stability_check(c, ga::light_angle_warns(&options.rules));
            ids.checks(core::slice::from_ref(&k));
            checks.push(json!({
                "name": k.name,
                "ok": k.ok,
                "message": k.message,
                "severity": k.severity,
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
            "lowest_gm_condition": a.lowest_gm().name,
            "limitations": a.limitations,
            "conditions": a
                .conditions
                .iter()
                .enumerate()
                .map(|(i, c)| with_trim(simulate_condition(c), a.trims.get(i)))
                .collect::<Vec<_>>(),
        });
    }
    let (curve_list, resistance) = curves(
        p,
        l,
        b,
        t,
        (
            hull_at(&fit_hull, t, rho)?,
            stern.map_or(0.0, |g| g.extra_wetted_surface(t)),
        ),
        disp,
        rho,
    )?;
    method_checks(resistance.as_ref(), &mut checks, &mut ids);
    let mut warnings = r.warnings.clone();
    if let Some(note) = channel_width_note(p)? {
        ids.text(&note);
        warnings.push(note);
    }
    let powering_block = powering_json(
        p,
        options.machinery.as_ref(),
        stern,
        rho,
        resistance.as_ref(),
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
    // Issue #450: the hull's own midship section, for the GA.
    drawing_parts.extend(midship_outline(&fit_hull));
    let drawing = drawing_json(drawing_parts);
    let air_loaded = r.air_draft_top_above_keel_m - t;
    with_blocks_and_text_ids(
        powering_block.clone(),
        sightline_json(r),
        with_loading_conditions(
        with_stern_mass(
        json!({
            "vessel": label,
            "cargo": cargo.name,
            "phase": cargo.phase.key(),
            "self_propelled": self_propelled,
            "dims": { "loa_m": num(l), "beam_m": num(b), "depth_m": num(d), "cb": num(laden_cb) },
            "hydrostatics": hydrostatics,
            "hydrostatic_table": hydrostatic_table,
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
                "cargo_t": num(cargo_t),
                "cargo_volume_bound_t": num(f.volume_capacity_t),
                "cargo_weight_bound_t": num(py_max(0.0, cb * l * b * t_max * rho - r.lightship_t)),
                "governs": governs,
                "cargo_volume_m3": num(r.cargo_volume_m3),
                "lightship_t": num(r.lightship_t),
                "displacement_t": num(disp),
                "deadweight_t": num(deadweight),
                "draught_m": num(t),
                "draught_max_m": num(t_max),
                "freeboard_m": num(d - t),
                "light_draught_m": opt_num(r.light_draft_m),
                "air_draft_loaded_m": num(air_loaded),
                "air_draft_light_m": opt_num(r.air_draft_light_m),
                "h_over_T": h_over_t(&ch, t)?,
            },
            "checks": checks,
            "warnings": warnings,
            "curves": curve_list,
            "stability": stability,
            "stability_marks": stability_marks,
            "cost_masses": r.gas_basis.as_ref().map_or(Value::Null, |b| match &powering_block {
                // Issue #353: the Option's own battery and Installed Power, not a tug's.
                Some(pw) => crate::cost::self_propelled_gas_masses(
                    &b.weights,
                    pw["battery_energy_kwh"].as_f64().unwrap_or_else(|| battery_units_kwh(p, r)),
                    pw["installed_power_kw"].as_f64().unwrap_or(0.0),
                ),
                None => crate::cost::gas_masses(&b.weights),
            }),
        }),
            (&r.stern_block_items, &r.trim_conditions),
        ),
            is_code_stability,
            loading_conditions,
        ),
        ids,
    )
    .map(|v| {
        (
            v,
            Measured {
                conditions,
                deadweight_t: Some(deadweight),
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
                vessel: Some(ovcd_core::vessel_model::VesselModelInputs {
                    capacity: f.engine_inputs.clone(),
                    result: f.engine_result.clone(),
                    hull: fit_hull.table.clone(),
                }),
                ..Measured::default()
            },
        )
    })
}

/// One stability condition as the simulator lists it: the fields it keeps, `fill` for the fill
/// fraction, and each criterion without anything extra.
fn simulate_condition(c: &ovcd_core::gas_stability::Condition) -> Value {
    let full = condition_json(c);
    let pick = |k: &str| full.get(k).cloned().unwrap_or(Value::Null);
    let mut condition = json!({
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
    });
    // Issue #592: the angle of down-flooding the criteria were worked with, only where there is
    // one, so a Design with none keeps its results (and hash).
    if let Some(a) = c.downflooding_deg {
        condition["downflooding_deg"] = json!(a);
    }
    condition
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

// --- Issue #426: the IS Code 2008 loading conditions ----------------------------------------------

/// Whether the Option asks for the IS Code 2008 loading conditions. A key left out keeps the
/// legacy conditions, so an old file's results do not move.
fn is_code(p: &Map<String, Value>) -> bool {
    p.get("loading_conditions").and_then(Value::as_str) == Some("is_code_2008")
}

/// The fuel use (l/h) at the service speed on the deep-water curve: the design speed, or the
/// maximum speed, interpolated between the curve's points (clamped to its ends).
fn service_fuel_lph(p: &Map<String, Value>, curves: &Value) -> R<f64> {
    let speed = match truthy_float(p, "design_speed_kn")? {
        Some(v) => v,
        None => get_or(p, "speed_max_kn", 10.0)?,
    };
    let Some(points) = curves
        .as_array()
        .and_then(|c| c.iter().find(|e| e["name"] == "Deep water"))
        .and_then(|e| e["points"].as_array())
    else {
        return Ok(0.0);
    };
    let pts: Vec<(f64, f64)> = points
        .iter()
        .filter_map(|q| Some((q["kn"].as_f64()?, q["fuel_lph"].as_f64()?)))
        .collect();
    let Some(&(first_kn, first_lph)) = pts.first() else {
        return Ok(0.0);
    };
    if speed <= first_kn {
        return Ok(first_lph);
    }
    for w in pts.windows(2) {
        let ((k0, f0), (k1, f1)) = (w[0], w[1]);
        if speed <= k1 {
            return Ok(f0 + (f1 - f0) * (speed - k0) / (k1 - k0));
        }
    }
    Ok(pts[pts.len() - 1].1)
}

/// The full stores and fuel (t) of the departure conditions. A pushed barge carries none: the
/// pusher carries them. A self-propelled vessel carries its fixed stores, and fuel for the
/// endurance at the service speed, unless it is battery-electric (its energy is in the lightship).
fn consumables_full_t(p: &Map<String, Value>, self_propelled: bool, curves: &Value) -> R<f64> {
    if !self_propelled {
        return Ok(0.0);
    }
    let stores = get_or(p, "fixed_stores_t", 0.0)?;
    let burns_fuel = !matches!(
        p.get("power_source").and_then(Value::as_str),
        None | Some("battery_electric")
    );
    let fuel = if burns_fuel {
        let density = TugInputs::default().fuel_density_kg_per_litre;
        service_fuel_lph(p, curves)? * get_or(p, "endurance_h", 0.0)? * density / 1000.0
    } else {
        0.0
    };
    Ok(stores + fuel)
}

/// The ballast plans of the two ballast conditions: the least ballast, or the user's fills by tank
/// group (the double bottom, the peaks, the wing tanks), both sides alike.
fn ballast_plans(
    p: &Map<String, Value>,
    tanks: &[ovcd_core::compartments::ClippedSpace],
) -> R<[ovcd_core::loading_conditions::BallastPlan; 2]> {
    use ovcd_core::loading_conditions::BallastPlan;
    Ok(match user_ballast(p)? {
        Some([departure, arrival]) => [departure.plan(tanks), arrival.plan(tanks)],
        None => [BallastPlan::Auto, BallastPlan::Auto],
    })
}

/// The user's ballast fills by tank group of the two ballast conditions, departure then arrival;
/// `None` when the plan is the least ballast.
fn user_ballast(
    p: &Map<String, Value>,
) -> R<Option<[ovcd_core::loading_conditions::GroupFills; 2]>> {
    use ovcd_core::loading_conditions::GroupFills;
    if p.get("ballast_plan").and_then(Value::as_str) != Some("user_set") {
        return Ok(None);
    }
    let plan = |condition: &str| -> R<GroupFills> {
        let fill = |group: &str| get_or(p, &format!("ballast_{condition}_{group}_fill"), 0.0);
        Ok(GroupFills {
            double_bottom: fill("double_bottom")?,
            peak: fill("peak")?,
            wing: fill("wing")?,
        })
    };
    Ok(Some([plan("departure")?, plan("arrival")?]))
}

/// The `may_sail_without_ballast` input; with none, a barge may and a self-propelled vessel may not.
fn may_sail_without_ballast(p: &Map<String, Value>, self_propelled: bool) -> bool {
    match p.get("may_sail_without_ballast") {
        Some(v) => truthy(v),
        None => !self_propelled,
    }
}

/// What the IS Code conditions of one Option need beyond its params.
struct IsCodeHull<'a> {
    dims: (f64, f64, f64),
    /// Issue #448: the hull's sections.
    hull: &'a Hull,
    rho: f64,
    criteria_set: &'a str,
    lightship: &'a [ovcd_core::loading_conditions::Item],
    full_cargo_t: f64,
    cargo_at: &'a dyn Fn(f64) -> Result<ovcd_core::loading_conditions::CargoAt, gs::PyError>,
    ballast_tanks: &'a [ovcd_core::compartments::ClippedSpace],
    consumables_t: f64,
    air_draft_top_m: f64,
    self_propelled: bool,
    /// The cargo masses the worst slack condition is searched over (liquid bulk); empty for none.
    slack_candidates_t: &'a [f64],
}

/// Assess the IS Code conditions.
fn is_code_conditions(
    p: &Map<String, Value>,
    h: &IsCodeHull,
) -> R<Vec<ovcd_core::loading_conditions::LoadedCondition>> {
    use ovcd_core::loading_conditions as lc;
    let (l, b, d) = h.dims;
    // The consumables tanks sit on the bottom at the aft end, in the stern block.
    let consumables = lc::Consumables {
        full_t: h.consumables_t,
        tanks: ovcd_core::compartments::consumables_tanks(b, 0.0, h.consumables_t),
        density: 1.0,
    };
    let may_sail = may_sail_without_ballast(p, h.self_propelled);
    let inputs = lc::Inputs {
        loa: l,
        beam: b,
        depth: d,
        hull: h.hull,
        water_density: h.rho,
        criteria_set: h.criteria_set,
        lightship: h.lightship,
        full_cargo_t: h.full_cargo_t,
        cargo_at: h.cargo_at,
        consumables,
        ballast_tanks: h.ballast_tanks,
        ballast_density: h.rho,
        ballast_plans: ballast_plans(p, h.ballast_tanks)?,
        may_sail_without_ballast: may_sail,
        air_draft_top_m: h.air_draft_top_m,
        air_draft_limit_m: truthy_float(p, "air_draft_limit_m")?,
    };
    let raised = |e: gs::PyError| Stop::Raised(e.type_name().to_string(), e.message().to_string());
    let mut conds = lc::assess(&inputs).map_err(raised)?;
    if !h.slack_candidates_t.is_empty() {
        conds.extend(lc::assess_slack(&inputs, h.slack_candidates_t).map_err(raised)?);
    }
    Ok(conds)
}

/// The ballast tanks of a capacity result, clipped by the hull (issue #455).
fn clipped_ballast_tanks(
    inp: &BargeCapacityInputs,
    r: &BargeCapacityResult,
    hull: &Hull,
) -> Vec<ovcd_core::compartments::ClippedSpace> {
    ovcd_core::compartments::ballast_tanks(inp, r)
        .iter()
        .map(|t| ovcd_core::compartments::clip(t, &hull.table))
        .collect()
}

/// The ballast tanks as the results list them: name, kind, side, capacity and centroid full.
fn ballast_tanks_json(tanks: &[ovcd_core::compartments::ClippedSpace], rho: f64) -> Value {
    Value::Array(
        tanks
            .iter()
            .map(|t| {
                let f = t.fill(1.0);
                json!({
                    "name": t.space.name,
                    "kind": t.space.kind.key(),
                    "side": t.space.side.key(),
                    "capacity_m3": num(t.volume_m3),
                    "capacity_t": num(t.capacity_t(rho)),
                    "lcg_m": num(f.lcg_m),
                    "tcg_m": num(f.tcg_m),
                    "vcg_m": num(f.vcg_m),
                })
            })
            .collect(),
    )
}

/// One IS Code condition in the results: the stability fields of a legacy condition, and its role,
/// consumables, ballast plan (quantity and disposition, IS Code 2008 Part B 3.4.2.3), freeboard,
/// air draft and trim.
fn loaded_condition_json(
    c: &ovcd_core::loading_conditions::LoadedCondition,
    with_weights: bool,
) -> Value {
    let mut v = with_trim(simulate_condition(&c.condition), Some(&c.trim));
    v["role"] = json!(c.role.key());
    v["consumables_t"] = num(c.consumables_t);
    v["ballast_t"] = num(c.ballast_t);
    v["ballast"] = Value::Array(
        c.ballast
            .iter()
            .map(|t| {
                json!({
                    "name": t.name,
                    "fill": num(t.fill),
                    "mass_t": num(t.mass_t),
                    "lcg_m": num(t.lcg_m),
                    "tcg_m": num(t.tcg_m),
                    "vcg_m": num(t.vcg_m),
                    "free_surface_moment_tm": num(t.free_surface_moment_tm),
                })
            })
            .collect(),
    );
    // Issue #332: a liquid bulk condition lists its weights and where they are.
    if with_weights {
        v["weights"] = Value::Array(
            c.weights
                .iter()
                .map(|(name, i)| {
                    json!({
                        "name": name,
                        "mass_t": num(i.mass_t),
                        "kcg_m": num(i.kcg_m),
                        "lcg_m": num(i.lcg_m),
                    })
                })
                .collect(),
        );
    }
    v["plan_found"] = c.plan_found.map_or(Value::Null, Value::Bool);
    v["freeboard_m"] = num(c.freeboard_m);
    v["air_draft_m"] = num(c.air_draft_m);
    v["air_draft_ok"] = c.air_draft_ok.map_or(Value::Null, Value::Bool);
    v["ok"] = json!(c.ok());
    v
}

/// The `loading_conditions` block of the results.
fn loading_conditions_json(
    conds: &[ovcd_core::loading_conditions::LoadedCondition],
    tanks: &[ovcd_core::compartments::ClippedSpace],
    consumables_t: f64,
    rho: f64,
    with_weights: bool,
) -> Value {
    let capacity_m3 = tanks.iter().fold(0.0, |a, t| a + t.volume_m3);
    json!({
        "set": "IMO IS Code 2008, Part B 3.4.1.2",
        "ok": conds.iter().all(|c| c.ok()),
        "consumables_t": num(consumables_t),
        "ballast_tank_count": tanks.len(),
        "ballast_capacity_m3": num(capacity_m3),
        "ballast_capacity_t": num(capacity_m3 * rho),
        "ballast_tanks": ballast_tanks_json(tanks, rho),
        "conditions": conds.iter().map(|c| loaded_condition_json(c, with_weights)).collect::<Vec<_>>(),
    })
}

/// The checks of the IS Code conditions: the stability of each, a ballast plan that meets every
/// criterion, and the air draft. `severity` is "error" for gas; "warning" for dry bulk, which has
/// no Rule Set of its own yet and whose low freeboard the seagoing criteria of IS Code 2008 Part A
/// 2.2 do not suit (as the hold GM check, engineering basis triage item 2).
fn loading_condition_checks(
    conds: &[ovcd_core::loading_conditions::LoadedCondition],
    severity: &str,
    light_angle_warns: bool,
    checks: &mut Vec<Value>,
    ids: &mut TextIds,
) {
    // Issue #498: a gas Design ("error") downgrades the angle of max GZ alone in the light condition;
    // a dry-bulk one (its checks are warnings) keeps its checks as they were.
    let listed: Vec<ovcd_core::checks::Check> = if severity == "error" {
        ga::loading_condition_checks_of(conds, light_angle_warns)
    } else {
        conds
            .iter()
            .flat_map(ga::loading_condition_checks)
            .collect()
    };
    for k in listed {
        ids.checks(core::slice::from_ref(&k));
        // A warning of the check stays a warning (issue #498: the angle of maximum GZ with no ballast).
        let severity = if k.severity == "warning" {
            "warning"
        } else {
            severity
        };
        checks.push(
            json!({ "name": k.name, "ok": k.ok, "message": k.message, "severity": severity }),
        );
    }
}

/// The `stability` block of the IS Code conditions, in the shape of the legacy gas one: the
/// lowest-GM condition is the one with the lowest fluid GM, the first of equal.
fn is_code_stability_json(
    conds: &[ovcd_core::loading_conditions::LoadedCondition],
    with_weights: bool,
    ids: &mut TextIds,
) -> Value {
    let mut lowest = &conds[0].condition;
    for c in &conds[1..] {
        if c.condition.gm_fluid_m < lowest.gm_fluid_m {
            lowest = &c.condition;
        }
    }
    let limitations = vec![
        text!(
            "stab-limit-concept-stage",
            "Concept-stage: the hull's sections, first-order weight heights, no downflooding openings."
        ),
        text!(
            "stab-limit-no-wind-or-damage",
            "No wind heel and no damage stability. Both are needed before approval: damage \
             stability by ADN 9.3.1.15 and IGC 2.7, wind heel by the IS Code 2008 weather \
             criterion (Part A 2.3)."
        ),
        text!(
            "stab-limit-moulded-tanks",
            "Ballast and consumables tanks are moulded volumes with no structure deduction; one tank \
             per side and kind, and the consumables tank height is an estimate."
        ),
    ];
    ids.texts(&limitations);
    json!({
        "criteria_set": "IMO IS Code 2008, Part A 2.2",
        "ok": conds.iter().all(|c| c.ok()),
        "lowest_gm_condition": lowest.name,
        "limitations": limitations,
        "conditions": conds.iter().map(|c| loaded_condition_json(c, with_weights)).collect::<Vec<_>>(),
    })
}

// --- Issue #359: what the Formation powering reads of the single-hull resistance ------------------

/// The resistance of an Equivalent Hull and its power curves, by the method of the single hull
/// (`resistance_of`, `powering_curves`). `Err` is the engine's refusal as text; `None` with no draught.
pub(crate) fn formation_curves(
    p: &Map<String, Value>,
    (l, b, t): (f64, f64, f64),
    cb: f64,
    disp: f64,
    rho: f64,
) -> Result<Option<Vec<ovcd_core::powering::EnvironmentCurve>>, String> {
    let stop_text = |s: Stop| match s {
        Stop::Raised(kind, message) => format!("{kind}: {message}"),
        Stop::Eval(e) => e.to_string(),
    };
    // An Equivalent Hull has no sections: its waterline length is its length, and Am = B x T x Cm.
    let cm = get_or(
        p,
        "midship_coefficient",
        ovcd_core::confined_water::DEFAULT_MIDSHIP_COEFFICIENT,
    )
    .map_err(stop_text)?;
    let hull = HullAt {
        cb,
        lwl: l,
        midship_m2: b * t * cm,
    };
    resistance_of(p, l, b, t, (hull, 0.0), disp, rho)
        .map(|r| r.map(|r| powering_curves(&r)))
        .map_err(stop_text)
}
