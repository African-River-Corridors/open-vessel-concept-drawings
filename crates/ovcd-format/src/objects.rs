//! Objects the Python caller hands to a function, read back from JSON.
//!
//! The package's result objects (`BargeCapacityResult`, `TankResult`, `WeightBreakdown`, the aft
//! and separation results) are plain data. A caller may build or edit one and pass it to
//! `journey_derating`, `gas_stability.assess`, `ga_adapter`, `weights` or `hydrostatics`. The
//! original Python used the object it was given, so these readers rebuild the core structs from
//! exactly the fields of that object. Nothing here recomputes a value.
//!
//! A field that is missing or of the wrong type is bad input (a Python `TypeError`). A float may
//! be the tagged text `"float:nan"`, `"float:inf"` or `"float:-inf"`.

use crate::EvalError;
use crate::eval::{barge_inputs_from, tagged_f64};
use ovcd_core::aft_arrangement::AftArrangementResult;
use ovcd_core::barge_capacity::{BargeCapacityResult, GasBasis};
use ovcd_core::checks::Check;
use ovcd_core::gas_separation::GasSeparationResult;
use ovcd_core::models::{TankResult, WeightBreakdown};
use ovcd_core::text::Text;

use serde_json::{Map, Value};

/// One JSON object with the name it is known by, for the error messages.
struct Obj<'a> {
    map: &'a Map<String, Value>,
    ctx: String,
}

impl<'a> Obj<'a> {
    fn new(value: &'a Value, ctx: &str) -> Result<Self, EvalError> {
        match value {
            Value::Object(map) => Ok(Self {
                map,
                ctx: ctx.to_string(),
            }),
            _ => Err(EvalError::BadInput(format!("`{ctx}` must be an object"))),
        }
    }

    fn bad(&self, name: &str, what: &str) -> EvalError {
        EvalError::BadInput(format!("`{}.{name}` must be {what}", self.ctx))
    }

    fn get(&self, name: &str) -> Result<&'a Value, EvalError> {
        self.map
            .get(name)
            .ok_or_else(|| EvalError::BadInput(format!("`{}.{name}` is missing", self.ctx)))
    }

    fn f64(&self, name: &str) -> Result<f64, EvalError> {
        tagged_f64(self.get(name)?).ok_or_else(|| self.bad(name, "a number"))
    }

    fn opt_f64(&self, name: &str) -> Result<Option<f64>, EvalError> {
        match self.get(name)? {
            Value::Null => Ok(None),
            v => tagged_f64(v)
                .map(Some)
                .ok_or_else(|| self.bad(name, "a number or null")),
        }
    }

    fn i64(&self, name: &str) -> Result<i64, EvalError> {
        self.get(name)?
            .as_i64()
            .ok_or_else(|| self.bad(name, "an integer"))
    }

    fn opt_i64(&self, name: &str) -> Result<Option<i64>, EvalError> {
        match self.get(name)? {
            Value::Null => Ok(None),
            v => v
                .as_i64()
                .map(Some)
                .ok_or_else(|| self.bad(name, "an integer or null")),
        }
    }

    fn string(&self, name: &str) -> Result<String, EvalError> {
        self.get(name)?
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| self.bad(name, "text"))
    }

    fn bool(&self, name: &str) -> Result<bool, EvalError> {
        self.get(name)?
            .as_bool()
            .ok_or_else(|| self.bad(name, "true or false"))
    }

    fn array(&self, name: &str) -> Result<&'a Vec<Value>, EvalError> {
        self.get(name)?
            .as_array()
            .ok_or_else(|| self.bad(name, "a list"))
    }

    fn f64_list(&self, name: &str) -> Result<Vec<f64>, EvalError> {
        self.array(name)?
            .iter()
            .map(|v| tagged_f64(v).ok_or_else(|| self.bad(name, "a list of numbers")))
            .collect()
    }

    fn string_list(&self, name: &str) -> Result<Vec<String>, EvalError> {
        self.array(name)?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| self.bad(name, "a list of text"))
            })
            .collect()
    }

    /// A list of text read back from a result: English with no Message Id.
    fn text_list(&self, name: &str) -> Result<Vec<Text>, EvalError> {
        Ok(self
            .string_list(name)?
            .into_iter()
            .map(Text::plain)
            .collect())
    }

    fn checks(&self, name: &str) -> Result<Vec<Check>, EvalError> {
        self.array(name)?
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let c = Obj::new(v, &format!("{}.{name}[{i}]", self.ctx))?;
                Ok(Check {
                    name: c.string("name")?,
                    ok: c.bool("ok")?,
                    message: c.string("message")?,
                    severity: c.string("severity")?,
                    msg: None,
                })
            })
            .collect()
    }
}

/// A `TankResult` from its fields.
pub(crate) fn tanks_from_json(value: &Value, ctx: &str) -> Result<TankResult, EvalError> {
    let o = Obj::new(value, ctx)?;
    Ok(TankResult {
        n_tanks_wide: o.i64("n_tanks_wide")?,
        n_tanks_long: o.i64("n_tanks_long")?,
        n_tanks_total: o.i64("n_tanks_total")?,
        outer_diameter: o.f64("outer_diameter")?,
        inner_diameter: o.f64("inner_diameter")?,
        overall_tank_length: o.f64("overall_tank_length")?,
        cylinder_length: o.f64("cylinder_length")?,
        head_length: o.f64("head_length")?,
        volume_per_tank: o.f64("volume_per_tank")?,
        volume_cylinder: o.f64("volume_cylinder")?,
        volume_heads: o.f64("volume_heads")?,
        total_tank_volume: o.f64("total_tank_volume")?,
        cargo_volume: o.f64("cargo_volume")?,
        tank_shell_weight: o.f64("tank_shell_weight")?,
        insulation_weight: o.f64("insulation_weight")?,
        tank_top_above_keel: o.f64("tank_top_above_keel")?,
        cylinder_wall_thickness: o.f64("cylinder_wall_thickness")?,
        head_wall_thickness: o.f64("head_wall_thickness")?,
        design_pressure: o.f64("design_pressure")?,
        tank_material: o.string("tank_material")?,
    })
}

/// A `WeightBreakdown` from its fields.
pub(crate) fn weights_from_json(value: &Value, ctx: &str) -> Result<WeightBreakdown, EvalError> {
    let o = Obj::new(value, ctx)?;
    Ok(WeightBreakdown {
        hull_bottom_plating: o.f64("hull_bottom_plating")?,
        hull_side_plating: o.f64("hull_side_plating")?,
        hull_deck_plating: o.f64("hull_deck_plating")?,
        hull_rake_plating: o.f64("hull_rake_plating")?,
        hull_internal_structure: o.f64("hull_internal_structure")?,
        hull_steel_total: o.f64("hull_steel_total")?,
        tank_shells: o.f64("tank_shells")?,
        tank_internals_supports: o.f64("tank_internals_supports")?,
        tank_total: o.f64("tank_total")?,
        insulation: o.f64("insulation")?,
        wing_plate: o.f64("wing_plate")?,
        weather_shield: o.f64("weather_shield")?,
        piping_machinery: o.f64("piping_machinery")?,
        outfitting: o.f64("outfitting")?,
        stores: o.f64("stores")?,
        moulded_volume: o.f64("moulded_volume")?,
        moulded_depth: o.f64("moulded_depth")?,
        steel_top_down: o.f64("steel_top_down")?,
        steel_bottom_up: o.f64("steel_bottom_up")?,
        steel_delta: o.f64("steel_delta")?,
        lightship: o.f64("lightship")?,
    })
}

fn aft_from_json(value: &Value, ctx: &str) -> Result<AftArrangementResult, EvalError> {
    let o = Obj::new(value, ctx)?;
    let zones = o
        .array("zones")?
        .iter()
        .enumerate()
        .map(|(i, z)| {
            let bad = || EvalError::BadInput(format!("`{ctx}.zones[{i}]` must be [label, x, x]"));
            match z.as_array().map(Vec::as_slice) {
                Some([label, a, f]) => Ok((
                    label.as_str().ok_or_else(bad)?.to_string(),
                    tagged_f64(a).ok_or_else(bad)?,
                    tagged_f64(f).ok_or_else(bad)?,
                )),
                _ => Err(bad()),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(AftArrangementResult {
        length_m: o.f64("length_m")?,
        zones,
        thruster_offsets_m: o.f64_list("thruster_offsets_m")?,
        thruster_room_length_m: o.f64("thruster_room_length_m")?,
        container_x0_m: o.f64("container_x0_m")?,
        container_x1_m: o.f64("container_x1_m")?,
        container_total_width_m: o.f64("container_total_width_m")?,
        container_top_above_keel_m: o.f64("container_top_above_keel_m")?,
        container_labels: o.string_list("container_labels")?,
        container_unit_width_m: o.f64("container_unit_width_m")?,
        container_side_gap_m: o.f64("container_side_gap_m")?,
        container_rows: o.i64("container_rows")?,
        container_per_row: o.i64("container_per_row")?,
        container_row_length_m: o.f64("container_row_length_m")?,
        container_row_gap_m: o.f64("container_row_gap_m")?,
        wheelhouse_x0_m: o.f64("wheelhouse_x0_m")?,
        wheelhouse_x1_m: o.f64("wheelhouse_x1_m")?,
        wheelhouse_floor_lowered_m: o.f64("wheelhouse_floor_lowered_m")?,
        wheelhouse_top_lowered_m: o.f64("wheelhouse_top_lowered_m")?,
        wheelhouse_floor_raised_m: o.f64("wheelhouse_floor_raised_m")?,
        wheelhouse_top_raised_m: o.f64("wheelhouse_top_raised_m")?,
        wheelhouse_lift_m: o.f64("wheelhouse_lift_m")?,
        wheelhouse_width_m: o.f64("wheelhouse_width_m")?,
        wheelhouse_height_m: o.f64("wheelhouse_height_m")?,
        block_top_lowered_m: o.f64("block_top_lowered_m")?,
        checks: o.checks("checks")?,
        warnings: o.text_list("warnings")?,
        // Issue #180 fields. A result saved before them is a battery-electric block.
        propulsion: opt_str(value, "propulsion", "battery_electric"),
        container_layout: opt_str(value, "container_layout", "rows"),
        container_units: match value.get("container_units") {
            None | Some(Value::Null) => Vec::new(),
            Some(_) => o
                .array("container_units")?
                .iter()
                .enumerate()
                .map(|(i, u)| {
                    let bad = || {
                        EvalError::BadInput(format!(
                            "`{ctx}.container_units[{i}]` must be [label, x, x, y]"
                        ))
                    };
                    match u.as_array().map(Vec::as_slice) {
                        Some([label, a, f, y]) => Ok((
                            label.as_str().ok_or_else(bad)?.to_string(),
                            tagged_f64(a).ok_or_else(bad)?,
                            tagged_f64(f).ok_or_else(bad)?,
                            tagged_f64(y).ok_or_else(bad)?,
                        )),
                        _ => Err(bad()),
                    }
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        engine_room_x0_m: opt_f64(value, "engine_room_x0_m"),
        engine_room_x1_m: opt_f64(value, "engine_room_x1_m"),
        gvu_room_x0_m: opt_f64(value, "gvu_room_x0_m"),
        gvu_room_x1_m: opt_f64(value, "gvu_room_x1_m"),
        n_gensets: value.get("n_gensets").and_then(Value::as_i64).unwrap_or(0),
        genset_model: opt_str(value, "genset_model", ""),
        genset_engine_model: opt_str(value, "genset_engine_model", ""),
        genset_kwe: opt_f64(value, "genset_kwe"),
        installed_kwe: opt_f64(value, "installed_kwe"),
        genset_offsets_m: match value.get("genset_offsets_m") {
            None | Some(Value::Null) => Vec::new(),
            Some(_) => o.f64_list("genset_offsets_m")?,
        },
        genset_x0_m: opt_f64(value, "genset_x0_m"),
        genset_length_m: opt_f64(value, "genset_length_m"),
        genset_width_m: opt_f64(value, "genset_width_m"),
        genset_height_m: opt_f64(value, "genset_height_m"),
        genset_engine_mass_t: opt_f64(value, "genset_engine_mass_t"),
        below_deck_length_m: opt_f64(value, "below_deck_length_m"),
        deck_length_m: match value.get("deck_length_m") {
            None | Some(Value::Null) => o.f64("length_m")?,
            Some(_) => o.f64("deck_length_m")?,
        },
    })
}

fn opt_str(value: &Value, key: &str, default: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_string()
}

fn opt_f64(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(tagged_f64).unwrap_or(0.0)
}

fn separation_from_json(value: &Value, ctx: &str) -> Result<GasSeparationResult, EvalError> {
    let o = Obj::new(value, ctx)?;
    Ok(GasSeparationResult {
        aft_separation_m: o.f64("aft_separation_m")?,
        fwd_separation_m: o.f64("fwd_separation_m")?,
        cofferdam_m: o.f64("cofferdam_m")?,
        service_space_m: o.f64("service_space_m")?,
        total_length_taken_m: o.f64("total_length_taken_m")?,
        checks: o.checks("checks")?,
        warnings: o.text_list("warnings")?,
    })
}

/// A `BargeCapacityResult` from its fields.
pub(crate) fn capacity_result_from_json(
    value: &Value,
    ctx: &str,
) -> Result<BargeCapacityResult, EvalError> {
    let o = Obj::new(value, ctx)?;
    let nested = |name: &str| -> Result<Option<&Value>, EvalError> {
        Ok(match o.get(name)? {
            Value::Null => None,
            v => Some(v),
        })
    };
    let gas_basis = match nested("gas_basis")? {
        None => None,
        Some(b) => {
            let bctx = format!("{ctx}.gas_basis");
            let b = Obj::new(b, &bctx)?;
            let inputs = barge_inputs_from(b.get("inputs")?)?;
            Some(GasBasis {
                inputs,
                tanks: tanks_from_json(b.get("tanks")?, &format!("{bctx}.tanks"))?,
                weights: weights_from_json(b.get("weights")?, &format!("{bctx}.weights"))?,
            })
        }
    };
    Ok(BargeCapacityResult {
        cargo_type: o.string("cargo_type")?,
        phase: o.string("phase")?,
        cargo_density_t_per_m3: o.f64("cargo_density_t_per_m3")?,
        cargo_capacity_t: o.f64("cargo_capacity_t")?,
        full_load_draft_m: o.f64("full_load_draft_m")?,
        air_draft_m: o.f64("air_draft_m")?,
        cargo_volume_m3: o.f64("cargo_volume_m3")?,
        lightship_t: o.f64("lightship_t")?,
        displacement_t: o.f64("displacement_t")?,
        deadweight_t: o.f64("deadweight_t")?,
        freeboard_m: o.f64("freeboard_m")?,
        air_draft_top_above_keel_m: o.f64("air_draft_top_above_keel_m")?,
        overloaded: o.bool("overloaded")?,
        tank_outer_diameter_m: o.opt_f64("tank_outer_diameter_m")?,
        n_tanks_wide: o.opt_i64("n_tanks_wide")?,
        max_tank_od_m: o.opt_f64("max_tank_od_m")?,
        n_tanks_total: o.opt_i64("n_tanks_total")?,
        side_clearance_m: o.opt_f64("side_clearance_m")?,
        bottom_clearance_m: o.opt_f64("bottom_clearance_m")?,
        hold_height_m: o.opt_f64("hold_height_m")?,
        hold_top_width_m: o.opt_f64("hold_top_width_m")?,
        hold_bottom_width_m: o.opt_f64("hold_bottom_width_m")?,
        hold_section_area_m2: o.opt_f64("hold_section_area_m2")?,
        gm_m: o.opt_f64("gm_m")?,
        bow_void_length_m: o.opt_f64("bow_void_length_m")?,
        stern_void_length_m: o.opt_f64("stern_void_length_m")?,
        cargo_zone_length_m: o.opt_f64("cargo_zone_length_m")?,
        bow_rake_length_m: o.opt_f64("bow_rake_length_m")?,
        aft_arrangement: nested("aft_arrangement")?
            .map(|v| aft_from_json(v, &format!("{ctx}.aft_arrangement")))
            .transpose()?,
        separation: nested("separation")?
            .map(|v| separation_from_json(v, &format!("{ctx}.separation")))
            .transpose()?,
        containment_inner_volume_m3: o.opt_f64("containment_inner_volume_m3")?,
        fill_ratio_applied: o.opt_f64("fill_ratio_applied")?,
        insulation_thickness_mm: o.opt_f64("insulation_thickness_mm")?,
        tank_wall_thickness_mm: o.opt_f64("tank_wall_thickness_mm")?,
        tank_inner_diameter_m: o.opt_f64("tank_inner_diameter_m")?,
        light_draft_m: o.opt_f64("light_draft_m")?,
        air_draft_light_m: o.opt_f64("air_draft_light_m")?,
        ballast_draft_m: o.opt_f64("ballast_draft_m")?,
        air_draft_ballast_m: o.opt_f64("air_draft_ballast_m")?,
        air_draft_navigating_light_m: o.opt_f64("air_draft_navigating_light_m")?,
        navigating_top_above_keel_m: o.opt_f64("navigating_top_above_keel_m")?,
        gas_basis,
        checks: o.checks("checks")?,
        warnings: o.text_list("warnings")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::capacity_json;
    use ovcd_core::barge_capacity::{BargeCapacityInputs, compute};

    /// A result written as JSON and read back is the same result, for a gas and a bulk barge.
    #[test]
    fn a_capacity_result_survives_a_round_trip() {
        for cargo in ["LPG_propane", "bauxite"] {
            let mut inputs = BargeCapacityInputs::new(90.0, 14.8, 6.5);
            inputs.cargo_type = cargo.to_string();
            let result = compute(&inputs).expect("computes");
            let back = capacity_result_from_json(&capacity_json(&result), "result").expect("reads");
            assert_eq!(back, result, "{cargo}");
        }
    }

    #[test]
    fn a_missing_field_is_bad_input_and_names_the_field() {
        let mut inputs = BargeCapacityInputs::new(90.0, 14.8, 6.5);
        inputs.cargo_type = "bauxite".to_string();
        let mut json = capacity_json(&compute(&inputs).expect("computes"));
        json.as_object_mut().expect("object").remove("lightship_t");
        let err = capacity_result_from_json(&json, "result").expect_err("refused");
        assert_eq!(
            err,
            EvalError::BadInput("`result.lightship_t` is missing".to_string())
        );
    }

    #[test]
    fn tagged_non_finite_numbers_are_read() {
        assert!(
            tagged_f64(&Value::String("float:nan".into()))
                .unwrap()
                .is_nan()
        );
        assert_eq!(
            tagged_f64(&Value::String("float:-inf".into())),
            Some(f64::NEG_INFINITY)
        );
        assert_eq!(tagged_f64(&Value::String("x".into())), None);
    }
}
