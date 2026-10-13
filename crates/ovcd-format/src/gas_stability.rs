//! JSON adapter for the `gas_stability` parity entry point: `box_gz`, `segment_fill`,
//! `criteria_set_for` and `assess`. It copies the frozen Python driver's output shapes
//! (`parity/driver.py`, `to_json`), including which numbers are ints.
//!
//! `assess` is not yet reachable through `eval`. The parity cases describe the arrangement to
//! assess by `fit` (`gas_arrangement.fit`) or `capacity` (`barge_capacity.compute`), and neither is
//! ported. Until both are, `assess` here takes the numbers it reads as `args.result` (see
//! `parity/gas_basis_fixtures.py` and `tests/gas_stability.rs`), and `gas_stability` stays out of
//! `parity/ported.json`. When `fit` lands: build `args.result` from the Rust `fit`, route
//! `eval("gas_stability", ..)` here, and add the entry point to `ported.json`.

use crate::EvalError;
use ovcd_core::gas_stability::{
    self as gs, Condition, Criterion, GasBasis, GasCapacityResult, PyError, PyNum,
};
use serde_json::{Map, Value, json};

/// A float as the Reference Results write it: NaN and infinity are tagged text.
fn float(x: f64) -> Value {
    if x.is_nan() {
        json!("float:nan")
    } else if x.is_infinite() {
        json!(if x > 0.0 { "float:inf" } else { "float:-inf" })
    } else {
        json!(x)
    }
}

fn num_arg(args: &Value, name: &str) -> Result<f64, EvalError> {
    args.get(name)
        .and_then(Value::as_f64)
        .ok_or_else(|| EvalError::BadInput(format!("`args.{name}` must be a number")))
}

fn opt_num_arg(args: &Value, name: &str, default: f64) -> Result<f64, EvalError> {
    match args.get(name) {
        None => Ok(default),
        Some(_) => num_arg(args, name),
    }
}

fn py_result(r: Result<Value, PyError>) -> Result<Value, EvalError> {
    Ok(match r {
        Ok(v) => json!({ "ok": v }),
        Err(e) => json!({ "error": { "type": e.type_name(), "message": e.message() } }),
    })
}

fn criterion_json(c: &Criterion) -> Value {
    let value = match c.value {
        PyNum::Int(i) => json!(i),
        PyNum::Float(x) => float(x),
    };
    json!({
        "name": c.name,
        "value": value,
        "limit": float(c.limit),
        "ok": c.ok,
        "source": c.source,
    })
}

/// A `Condition` as `to_json` writes it. When the barge does not float (`gz` is empty), Python
/// stores the int `-1` or `0` in the fields it does not compute.
fn condition_json(c: &Condition) -> Value {
    let floats = !c.gz.is_empty();
    let fl_or_int = |x: f64| if floats { float(x) } else { json!(x as i64) };
    let mut m = Map::new();
    m.insert("name".into(), json!(c.name));
    m.insert("cargo_t".into(), float(c.cargo_t));
    m.insert("fill_fraction".into(), float(c.fill_fraction));
    m.insert("displacement_t".into(), float(c.displacement_t));
    m.insert("draught_m".into(), float(c.draught_m));
    m.insert("kg_m".into(), float(c.kg_m));
    m.insert("gm_solid_m".into(), fl_or_int(c.gm_solid_m));
    m.insert("free_surface_m".into(), fl_or_int(c.free_surface_m));
    m.insert("gm_fluid_m".into(), fl_or_int(c.gm_fluid_m));
    m.insert(
        "gz".into(),
        Value::Array(c.gz.iter().map(|&(d, g)| json!([d, float(g)])).collect()),
    );
    m.insert("max_gz_m".into(), fl_or_int(c.max_gz_m));
    m.insert("max_gz_deg".into(), json!(c.max_gz_deg));
    m.insert("area_0_27".into(), fl_or_int(c.area_0_27));
    m.insert("area_0_30".into(), fl_or_int(c.area_0_30));
    m.insert("area_0_40".into(), fl_or_int(c.area_0_40));
    m.insert("area_30_40".into(), fl_or_int(c.area_30_40));
    m.insert(
        "deck_edge_deg".into(),
        c.deck_edge_deg.map_or(Value::Null, float),
    );
    m.insert(
        "criteria".into(),
        Value::Array(c.criteria.iter().map(criterion_json).collect()),
    );
    Value::Object(m)
}

fn gas_basis(v: &Value) -> Result<GasBasis, EvalError> {
    let n = u32::try_from(
        v.get("n_tanks_total")
            .and_then(Value::as_u64)
            .ok_or_else(|| EvalError::BadInput("`n_tanks_total` must be a whole number".into()))?,
    )
    .map_err(|_| EvalError::BadInput("`n_tanks_total` is too large".into()))?;
    Ok(GasBasis {
        loa: num_arg(v, "loa")?,
        beam: num_arg(v, "beam")?,
        depth: num_arg(v, "depth")?,
        double_bottom_height: num_arg(v, "double_bottom_height")?,
        tank_support_height: num_arg(v, "tank_support_height")?,
        outer_diameter: num_arg(v, "outer_diameter")?,
        inner_diameter: num_arg(v, "inner_diameter")?,
        total_tank_volume: num_arg(v, "total_tank_volume")?,
        n_tanks_total: n,
        cylinder_length: num_arg(v, "cylinder_length")?,
        steel_top_down: num_arg(v, "steel_top_down")?,
        tank_total: num_arg(v, "tank_total")?,
        insulation: num_arg(v, "insulation")?,
        piping_machinery: num_arg(v, "piping_machinery")?,
        outfitting: num_arg(v, "outfitting")?,
    })
}

fn capacity_result(v: &Value) -> Result<GasCapacityResult, EvalError> {
    let basis = match v.get("gas_basis") {
        None | Some(Value::Null) => None,
        Some(b) => Some(gas_basis(b)?),
    };
    Ok(GasCapacityResult {
        gas_basis: basis,
        cargo_density_t_per_m3: num_arg(v, "cargo_density_t_per_m3")?,
        lightship_t: num_arg(v, "lightship_t")?,
        has_aft_arrangement: v
            .get("has_aft_arrangement")
            .and_then(Value::as_bool)
            .ok_or_else(|| {
                EvalError::BadInput("`has_aft_arrangement` must be true or false".into())
            })?,
    })
}

/// Run one `gas_stability` function on case-shaped arguments and return `{"ok": ..}` or
/// `{"error": ..}`.
pub fn run(function: &str, args: &Value) -> Result<Value, EvalError> {
    match function {
        "box_gz" => py_result(
            gs::box_gz(
                num_arg(args, "beam")?,
                num_arg(args, "depth")?,
                num_arg(args, "draught")?,
                num_arg(args, "kg")?,
                num_arg(args, "phi_deg")?,
            )
            .map(float),
        ),
        "segment_fill" => py_result(
            gs::segment_fill(num_arg(args, "radius")?, num_arg(args, "volume_fraction")?)
                .map(|(h, y)| json!([float(h), float(y)])),
        ),
        "criteria_set_for" => {
            let ruleset = args
                .get("ruleset")
                .and_then(Value::as_str)
                .ok_or_else(|| EvalError::BadInput("`args.ruleset` must be a string".into()))?;
            py_result(Ok(json!(gs::criteria_set_for(ruleset))))
        }
        "assess" => {
            let result = capacity_result(
                args.get("result")
                    .ok_or_else(|| EvalError::BadInput("`args.result` is required".into()))?,
            )?;
            let payload = num_arg(args, "payload_t")?;
            let cb = opt_num_arg(args, "block_coefficient", gs::DEFAULT_BLOCK_COEFFICIENT)?;
            let rho = opt_num_arg(args, "water_density", gs::DEFAULT_WATER_DENSITY)?;
            let cset = match args.get("criteria_set") {
                None => gs::DEFAULT_CRITERIA_SET,
                Some(v) => v.as_str().ok_or_else(|| {
                    EvalError::BadInput("`args.criteria_set` must be a string".into())
                })?,
            };
            py_result(gs::assess(&result, payload, cb, rho, cset).map(|a| {
                json!({
                    "criteria_set": a.criteria_set,
                    "conditions": a.conditions.iter().map(condition_json).collect::<Vec<_>>(),
                    "limitations": a.limitations,
                })
            }))
        }
        other => Err(EvalError::BadInput(format!(
            "gas_stability has no parity function `{other}`"
        ))),
    }
}
