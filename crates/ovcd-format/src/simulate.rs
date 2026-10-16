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
use crate::eval::{checks_json, num, ok, raised};
use crate::gas_stability::condition_json;
use ovcd_core::barge_capacity::{self, BargeCapacityInputs, BargeCapacityResult};
use ovcd_core::cargo::{CARGO_DATABASE, CargoProperties};
use ovcd_core::checks::Check;
use ovcd_core::gas_arrangement::{self as ga, DEFAULT_DEPTHS_M, FitOptions, RULESETS};
use ovcd_core::gas_stability::{self as gs, GasCapacityResult};
use ovcd_core::pyfmt::{fixed, py_str};
use ovcd_core::pystr::{parse_float, repr_str};
use ovcd_core::resistance::{self, EnvironmentInputs, ResistanceInputs, TugInputs};
use serde_json::{Map, Value, json};

/// Water density of the simulator (fresh water, t/m3).
const RHO: f64 = 1.0;
const EM_DASH: &str = "\u{2014}";
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
                    "phase": c.phase,
                    "density_kg_m3": num(c.density),
                })
            })
            .collect(),
    )
}

/// The `VESSELS` table: (label, hold type for the core, allowed phase).
fn vessel(id: &str) -> R<(&'static str, Option<&'static str>, &'static str)> {
    Ok(match id {
        "box_barge" => ("Box-hold dry bulk barge", Some("box"), "dry_bulk"),
        "hopper_barge" => (
            "Sloped-hopper dry bulk barge",
            Some("sloped_hopper"),
            "dry_bulk",
        ),
        "gas_barge" => ("Gas barge, IMO Type C tanks", None, "liquid_gas"),
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
        water_density: RHO,
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
fn curves(p: &Map<String, Value>, l: f64, b: f64, t: f64, cb: f64, disp: f64) -> R<Value> {
    let ch = channel(p)?;
    let mut envs = vec![env("Deep water", 50.0, 1000.0, 3.0, 0.0, true)];
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

fn air_draft_check(r: &BargeCapacityResult, limit: f64) -> R<Value> {
    let Some(light) = r.air_draft_light_m else {
        return bad("the capacity result has no light air draft");
    };
    Ok(json!({
        "name": "Air draft (empty, governing)",
        "ok": light <= limit,
        "message": format!("{} m vs limit {} m", fixed(light, 2), fixed(limit, 2)),
    }))
}

fn opt_int(x: Option<i64>) -> Value {
    x.map_or(Value::Null, |i| json!(i))
}

fn opt_num(x: Option<f64>) -> Value {
    x.map_or(Value::Null, num)
}

/// `simulate(params)`.
fn simulate(p: &Map<String, Value>) -> R<Value> {
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
    let cb = need(p, "cb")?;
    let self_propelled = p.get("self_propelled").is_some_and(truthy);
    if vessel_id == "gas_barge" {
        return simulate_gas(p, label, cargo, cargo_key, (l, b, d), cb, self_propelled);
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
    inp.required_freeboard = freeboard;
    inp.air_draft_limit_m = air_draft_limit;
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
    let disp_max = cb * l * b * t_max * RHO;
    let cargo_weight_bound = py_max(0.0, disp_max - ls);
    let cargo_volume_bound = r.cargo_capacity_t;
    let cargo_t = py_min(cargo_volume_bound, cargo_weight_bound);
    let governs = if cargo_volume_bound <= cargo_weight_bound {
        "hold volume".to_string()
    } else {
        format!("draught {EM_DASH} {lim_name}")
    };
    let disp = ls + cargo_t;
    let t = py_div(disp, cb * l * b * RHO)?;
    let h_top = r.air_draft_top_above_keel_m;

    let mut checks = vec![json!({
        "name": "Cargo > 0",
        "ok": cargo_t > 0.0,
        "message": if cargo_t <= 0.0 { "Lightship alone exceeds the allowed draught." } else { "OK" },
    })];
    capacity_checks(&r, &mut checks);
    if let Some(limit) = truthy_float(p, "air_draft_limit_m")? {
        checks.push(air_draft_check(&r, limit)?);
    }
    let curve_list = curves(p, l, b, t, cb, disp)?;
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
    Ok(json!({
        "vessel": label,
        "cargo": cargo.name,
        "phase": cargo.phase,
        "self_propelled": self_propelled,
        "arrangement": Value::Null,
        "dims": { "loa_m": num(l), "beam_m": num(b), "depth_m": num(d), "cb": num(cb) },
        "geometry": geo,
        "results": {
            "cargo_t": num(cargo_t),
            "cargo_volume_bound_t": num(cargo_volume_bound),
            "cargo_weight_bound_t": num(cargo_weight_bound),
            "governs": governs,
            "cargo_volume_m3": num(r.cargo_volume_m3),
            "lightship_t": num(ls),
            "displacement_t": num(disp),
            "deadweight_t": num(disp - ls),
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
    }))
}

/// Tank filling limit by ruleset: IGC 15.3 default 98 %, for both IGC rulesets.
fn fill_by_ruleset(_ruleset: &str) -> f64 {
    0.98
}

fn simulate_gas(
    p: &Map<String, Value>,
    label: &str,
    cargo: &CargoProperties,
    cargo_key: &str,
    (l, b, d0): (f64, f64, f64),
    cb: f64,
    self_propelled: bool,
) -> R<Value> {
    let ruleset = match p.get("ruleset") {
        None => "IGC_2G_EXACT",
        Some(Value::String(s)) => s.as_str(),
        // Python tests `ruleset not in RULESETS`, so any other value is an unknown ruleset.
        Some(other) => {
            return Ok(error_result(format!("Unknown ruleset {}.", py_repr(other))));
        }
    };
    if !RULESETS.contains(&ruleset) {
        return Ok(error_result(format!(
            "Unknown ruleset {}.",
            repr_str(ruleset)
        )));
    }
    let fill = match truthy_float(p, "fill_ratio")? {
        Some(v) => v,
        None => fill_by_ruleset(ruleset),
    };
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
        fill_ratio: fill,
        self_propelled,
        depths: depths.clone(),
        block_coefficient: cb,
        max_tank_od_m: get_or(
            p,
            "max_tank_od_m",
            ovcd_core::barge_capacity::DEFAULT_MAX_TANK_OD_M,
        )?,
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
    let governs = match f.binding.as_str() {
        "volume" => "tank volume".to_string(),
        "draft" => format!("draught {EM_DASH} {lim_name}"),
        "no cargo area" => "no cargo area (hull shorter than its fixed end spaces)".to_string(),
        other => return bad(format!("unknown binding `{other}`")),
    };
    let mut checks = vec![json!({
        "name": "Cargo > 0",
        "ok": f.payload_t > 0.0,
        "message": if f.payload_t > 0.0 { "OK" } else { "Nothing fits: lengthen or widen the hull." },
    })];
    if f.payload_t > 0.0 && !f.feasible {
        checks.push(json!({
            "name": "Feasible arrangement",
            "ok": false,
            "message": "No tank arrangement passes its checks on this hull; the best failing one is shown. See the failed checks.",
        }));
    }
    capacity_checks(r, &mut checks);
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
        checks.push(air_draft_check(r, limit)?);
    }
    let mut stability = Value::Null;
    if f.payload_t > 0.0 && r.gas_basis.is_some() {
        let basis = GasCapacityResult::try_from(r)
            .map_err(|m| Stop::Eval(EvalError::BadInput(m.into())))?;
        let a = gs::assess(
            &basis,
            f.payload_t,
            cb,
            gs::DEFAULT_WATER_DENSITY,
            gs::criteria_set_for(ruleset),
        )
        .map_err(|e| Stop::Raised(e.type_name().to_string(), e.message().to_string()))?;
        for c in &a.conditions {
            let bad_criteria: Vec<_> = c.criteria.iter().filter(|x| !x.ok).collect();
            let mut message = format!(
                "GM {} m (fluid), max GZ {} m at {}\u{b0}",
                fixed(c.gm_fluid_m, 2),
                fixed(c.max_gz_m, 2),
                fixed(f64::from(c.max_gz_deg), 0)
            );
            if !bad_criteria.is_empty() {
                let parts: Vec<String> = bad_criteria
                    .iter()
                    .map(|x| {
                        let v = match x.value {
                            gs::PyNum::Int(i) => i as f64,
                            gs::PyNum::Float(v) => v,
                        };
                        format!("{} {} < {}", x.name, fixed(v, 3), py_str(x.limit))
                    })
                    .collect();
                message.push_str(" \u{2014} fails: ");
                message.push_str(&parts.join(", "));
            }
            checks.push(json!({
                "name": format!("Stability \u{2014} {}", c.name.to_lowercase()),
                "ok": bad_criteria.is_empty(),
                "message": message,
            }));
        }
        let set_label = match a.criteria_set.as_str() {
            "IS_CODE_2008" => "IMO IS Code 2008, Part A 2.2",
            other => return bad(format!("unknown criteria set `{other}`")),
        };
        stability = json!({
            "criteria_set": set_label,
            "ok": a.ok(),
            "governing": a.governing().name,
            "limitations": a.limitations,
            "conditions": a.conditions.iter().map(simulate_condition).collect::<Vec<_>>(),
        });
    }
    let curve_list = curves(p, l, b, t, cb, disp)?;
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
    let air_loaded = r.air_draft_top_above_keel_m - t;
    Ok(json!({
        "vessel": label,
        "cargo": cargo.name,
        "phase": cargo.phase,
        "self_propelled": self_propelled,
        "dims": { "loa_m": num(l), "beam_m": num(b), "depth_m": num(d), "cb": num(cb) },
        "geometry": geo,
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
            "cargo_weight_bound_t": num(py_max(0.0, cb * l * b * t_max * RHO - r.lightship_t)),
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
    }))
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

/// `simulate(params)` as the app calls it: params in, an [`Outcome`] out.
pub fn simulate_params(params: &Map<String, Value>) -> Outcome {
    match simulate(params) {
        Ok(v) => Outcome::Result(v),
        Err(Stop::Raised(kind, message)) => Outcome::Raised(kind, message),
        Err(Stop::Eval(e)) => Outcome::Refused(e.to_string()),
    }
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
