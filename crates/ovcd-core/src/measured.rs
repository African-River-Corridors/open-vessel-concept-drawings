//! The typed values one computed design gives the Rule Trace, and the computed Measures read from
//! them (spec 0004, "Measure (code)"). A run (`ovcd-format`) and the arrangement search (`fit`) both
//! read Measures here, so a Rule gives the same verdict on a candidate as on the result.
//! Input Measures (`input.<name>`) are the caller's: it knows the inputs.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::barge_capacity::TankRuleValues;
use crate::box_stability::{Condition, area};
use crate::compartments::SpaceKind;
use crate::gas_separation::GasSeparationResult;
use crate::liquid_bulk::LiquidBulkResult;
use crate::rule_trace::{MeasureValue, ObjectType};
use crate::rules::{ArgValue, MeasureRef};

/// The typed values of one design that the Rule Trace reads: never JSON, so a Measure gives exactly
/// the number the engine worked with.
#[derive(Debug, Clone, Default)]
pub struct Measured {
    /// The gas stability loading conditions, in the order the engine builds them. Empty for a dry
    /// bulk barge (its stability is first-order only) and for a gas barge that carries nothing.
    pub conditions: Vec<Condition>,
    /// The deadweight, t: for the derived input `deadweight_band`. `None` when there is none.
    pub deadweight_t: Option<f64>,
    /// The Type C tank values the IGC Rules read; `None` but for a gas barge.
    pub tank: Option<TankRuleValues>,
    /// The gas cargo-area separation and whether its tanks have plane ends; `None` but for a gas
    /// barge.
    pub separation: Option<(GasSeparationResult, bool)>,
    /// The liquid bulk layout (spec 0006), for the liquid Measures; `None` but for a liquid bulk Cargo.
    /// The Measures read it and nothing else of the module.
    pub liquid: Option<LiquidBulkResult>,
    /// The design's check list, (name, ok), in order, with the names as a result lists them (spaces,
    /// not underscores): the verdicts the `ovcd` Rules mirror.
    pub checks: Vec<(String, bool)>,
}

/// The deadweight bands, at the thresholds of MARPOL Annex I Reg. 19 (MEPC.117(52)): 19.1, 600
/// tonnes deadweight and above; 19.2 and 19.6, 5,000 tonnes; 19.3.4, 20,000 tonnes.
pub const DEADWEIGHT_BANDS: &[&str] = &[
    "under-600-t",
    "600-to-5000-t",
    "5000-to-20000-t",
    "20000-t-and-above",
];

/// The band of a deadweight, t. "x tonnes deadweight and above" is in the band that starts at x.
pub fn deadweight_band(dw_t: f64) -> &'static str {
    if dw_t < 600.0 {
        DEADWEIGHT_BANDS[0]
    } else if dw_t < 5000.0 {
        DEADWEIGHT_BANDS[1]
    } else if dw_t < 20000.0 {
        DEADWEIGHT_BANDS[2]
    } else {
        DEADWEIGHT_BANDS[3]
    }
}

fn arg(m: &MeasureRef, name: &str) -> Result<f64, String> {
    match m.args.iter().find(|(k, _)| k == name) {
        Some((_, ArgValue::Number(x))) => Ok(*x),
        _ => Err(format!("`{}` needs the number argument `{name}`", m.id)),
    }
}

/// A heel argument: a whole number of degrees on the 0 to 60 degree grid.
fn deg(m: &MeasureRef, name: &str) -> Result<i32, String> {
    let x = arg(m, name)?;
    if libm::trunc(x) == x && (0.0..=60.0).contains(&x) {
        Ok(x as i32)
    } else {
        Err(format!(
            "`{}` argument `{name}` must be a whole number of degrees from 0 to 60",
            m.id
        ))
    }
}

impl Measured {
    /// The value of a liquid bulk Measure, or `None` for a Measure that is not one. Read from the
    /// [`LiquidBulkResult`] only (spec 0006 section 5.3).
    fn liquid_value(&self, m: &MeasureRef, index: usize) -> Option<Result<MeasureValue, String>> {
        let known = [
            "hull.double_side_width",
            "hull.double_bottom_height",
            "hull.lbh",
            "cargo-tank.volume",
            "cargo-tank.length",
            "option.cargo_capacity_m3",
            "option.slop_capacity_share",
        ];
        if !known.contains(&m.id.as_str()) {
            return None;
        }
        let Some(l) = self.liquid.as_ref() else {
            return Some(Err("the Option has no liquid bulk tank layout".to_string()));
        };
        let tank = || {
            l.of_kind(SpaceKind::CargoTank)
                .nth(index)
                .ok_or_else(|| format!("no cargo tank {index}"))
        };
        let number = |x: f64| Ok(MeasureValue::Number(x));
        Some(match m.id.as_str() {
            "hull.double_side_width" => number(l.double_side_width_m),
            "hull.double_bottom_height" => number(l.double_bottom_height_m),
            "hull.lbh" => number(l.lbh_m3),
            "cargo-tank.volume" => tank().and_then(|t| number(t.volume_m3())),
            "cargo-tank.length" => tank().and_then(|t| number(t.length())),
            "option.cargo_capacity_m3" => number(l.cargo_gross_m3),
            _ => number(l.slop_share_actual),
        })
    }

    /// The labels of the design's objects of one type. `option` names the one Option object.
    pub fn objects(&self, object: ObjectType, option: &str) -> Vec<String> {
        match object {
            ObjectType::Option => alloc::vec![option.to_string()],
            ObjectType::LoadingCondition => {
                self.conditions.iter().map(|c| c.name.clone()).collect()
            }
            ObjectType::Tank => match &self.tank {
                Some(t) => (1..=t.n_tanks.max(0))
                    .map(|i| format!("Tank {i}"))
                    .collect(),
                None => Vec::new(),
            },
            // Issue #337: the one hull of a liquid bulk layout, and its tanks by role.
            ObjectType::Hull => match self.liquid {
                Some(_) => alloc::vec!["Hull".to_string()],
                None => Vec::new(),
            },
            ObjectType::CargoTank => self.liquid_names(|k| k == SpaceKind::CargoTank),
            ObjectType::BallastTank => self.liquid_names(SpaceKind::is_ballast),
            ObjectType::SlopTank => self.liquid_names(|k| k == SpaceKind::Slop),
        }
    }

    /// The names of the liquid bulk Spaces of the kinds `keep` allows, in layout order.
    fn liquid_names(&self, keep: impl Fn(SpaceKind) -> bool) -> Vec<String> {
        self.liquid.as_ref().map_or_else(Vec::new, |l| {
            l.spaces
                .iter()
                .filter(|s| keep(s.kind))
                .map(|s| s.name.clone())
                .collect()
        })
    }

    /// The labels of the design's objects of the type with this key (as the kernel names it); none
    /// for a key OVCD does not declare.
    pub fn objects_by_key(&self, key: &str, option: &str) -> Vec<String> {
        ObjectType::from_key(key).map_or_else(Vec::new, |o| self.objects(o, option))
    }

    /// Whether the design has this Measure at all. A Rule on a Measure it lacks (a check the engine
    /// did not run for it) is `not applicable`, never a failure.
    pub fn has(&self, m: &MeasureRef) -> bool {
        match m.id.as_str() {
            "check.ok" => m.args.iter().any(|(k, v)| {
                k == "name"
                    && matches!(v, ArgValue::Text(n) if self.checks.iter().any(|(c, _)| c == n))
            }),
            id if id.starts_with("separation.") => self.separation.is_some(),
            // Issue #337: the liquid Measures exist for a liquid bulk layout only.
            "hull.double_side_width"
            | "hull.double_bottom_height"
            | "hull.lbh"
            | "cargo-tank.volume"
            | "cargo-tank.length"
            | "option.cargo_capacity_m3"
            | "option.slop_capacity_share" => self.liquid.is_some(),
            "option.deadweight" => self.deadweight_t.is_some(),
            _ => true,
        }
    }

    /// The value of a computed Measure on object `index` of its type, or why it cannot be computed.
    pub fn value(&self, m: &MeasureRef, index: usize) -> Result<MeasureValue, String> {
        if m.id == "check.ok" {
            let name = match m.args.iter().find(|(k, _)| k == "name") {
                Some((_, ArgValue::Text(n))) => n,
                _ => return Err("`check.ok` needs the text argument `name`".to_string()),
            };
            return self
                .checks
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, ok)| MeasureValue::Bool(*ok))
                .ok_or_else(|| format!("the result has no check `{name}`"));
        }
        if let Some(v) = self.liquid_value(m, index) {
            return v;
        }
        if m.id == "option.deadweight" {
            return self
                .deadweight_t
                .map(MeasureValue::Number)
                .ok_or_else(|| "the run gives no deadweight".to_string());
        }
        if let Some(name) = m.id.strip_prefix("separation.") {
            let (s, plane) = self
                .separation
                .as_ref()
                .ok_or_else(|| "the Option has no gas cargo-area separation".to_string())?;
            return match name {
                "cofferdam" => Ok(MeasureValue::Number(s.cofferdam_m)),
                "aft" => Ok(MeasureValue::Number(s.aft_separation_m)),
                "fwd" => Ok(MeasureValue::Number(s.fwd_separation_m)),
                "tank_end" => Ok(MeasureValue::Text(
                    if *plane { "plane" } else { "domed" }.to_string(),
                )),
                other => Err(format!("no value for Measure `separation.{other}`")),
            };
        }
        if let Some(name) = m.id.strip_prefix("tank.") {
            let t = self
                .tank
                .as_ref()
                .ok_or_else(|| "the Option has no Type C tanks".to_string())?;
            let number = |x: f64| Ok(MeasureValue::Number(x));
            return match name {
                "vc" => number(t.vc_m3),
                "side_gap" => number(t.side_gap_m),
                "bottom_gap" => number(t.bottom_gap_m),
                "insulation" => number(t.insulation_m),
                // The thinner wall; the first of equal, as Python's `min`.
                "wall_thickness" => number(if t.head_wall_mm < t.cylinder_wall_mm {
                    t.head_wall_mm
                } else {
                    t.cylinder_wall_mm
                }),
                "weld_factor" => number(t.weld_factor),
                "allowable_stress" => number(t.allowable_stress_mpa),
                "rm" => number(t.tensile_strength_mpa),
                "re" => number(t.yield_strength_mpa),
                "material" => Ok(MeasureValue::Text(t.material.clone())),
                "design_pressure" => number(t.design_pressure_mpa),
                // IGC 4.23.1.2: sigma_m is taken as the allowable stress f, the most the wall may
                // carry; the tank dimension C and the relative density are the tank's own.
                "dynamic_pressure_term" => {
                    number(crate::pressure_vessel::igc_dynamic_pressure_term_mpa(
                        t.allowable_stress_mpa,
                        t.outer_diameter_m,
                        t.overall_length_m,
                        t.liquid_density_t_per_m3,
                    ))
                }
                "design_temperature" => t
                    .design_temperature_c
                    .map(MeasureValue::Number)
                    .ok_or_else(|| "the design gives no tank design temperature".to_string()),
                other => Err(format!("no value for Measure `tank.{other}`")),
            };
        }
        let c = self
            .conditions
            .get(index)
            .ok_or_else(|| format!("no loading condition {index}"))?;
        if m.id == "stability.condition_ok" {
            // The check the engine lists for the condition (`gas_arrangement::stability_check`).
            let name = format!("Stability \u{2014} {}", c.name.to_lowercase());
            return self
                .checks
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, ok)| MeasureValue::Bool(*ok))
                .ok_or_else(|| format!("the result has no check `{name}`"));
        }
        if c.gz.is_empty() {
            return Err(format!(
                "{}: the barge does not float with freeboard, so it has no GZ curve",
                c.name
            ));
        }
        match m.id.as_str() {
            "gz.area" => {
                let (from, to) = (deg(m, "from_deg")?, deg(m, "to_deg")?);
                if from >= to {
                    return Err("`gz.area` needs from_deg below to_deg".to_string());
                }
                Ok(MeasureValue::Number(area(&c.gz, from, to)))
            }
            "gz.max_from" => {
                let from = deg(m, "from_deg")?;
                // `max()` keeps the first of equal values, as the IS Code criterion does.
                let mut best: Option<f64> = None;
                for &(d, g) in &c.gz {
                    if d >= from && best.is_none_or(|b| g > b) {
                        best = Some(g);
                    }
                }
                best.map(MeasureValue::Number)
                    .ok_or_else(|| format!("no GZ at {from} degrees or more"))
            }
            "gz.angle_of_max" => Ok(MeasureValue::Number(f64::from(c.max_gz_deg))),
            "gm0" => Ok(MeasureValue::Number(c.gm_fluid_m)),
            other => Err(format!("no value for Measure `{other}`")),
        }
    }
}
