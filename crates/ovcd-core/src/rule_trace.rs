//! OVCD's side of the rules engine (spec 0004, ADR 0013, ADR 0019): the object types, the Measure
//! registry and the formula shapes it registers with the domain-free kernel, `arc-rules`. The
//! evaluator and the Rule Trace are the kernel's ([`arc_rules::eval`]); they are re-exported here,
//! and [`evaluate`] and [`failures`] pass OVCD's formula shapes.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub use arc_rules::domain::{ArgDef, FormulaShape, MeasureInfo, MethodStatus, ValueKind};
pub use arc_rules::eval::{
    Failure, MeasureSource, MeasureValue, Outcome, ROW_SHAPES, Trace, TraceRow, UsedValue, applies,
    applies_to_object, holds, margin, row_shape, stricter, table_value,
};

use crate::rules::RuleSet;

/// The type of object a Measure is computed for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    Option,
    Hull,
    /// A Type C gas cargo tank.
    Tank,
    LoadingCondition,
    /// A liquid bulk cargo tank (spec 0006): one object type per tank role, so a Rule on cargo tanks
    /// never reaches a ballast or slop tank.
    CargoTank,
    BallastTank,
    SlopTank,
}

impl ObjectType {
    pub const ALL: [ObjectType; 7] = [
        ObjectType::Option,
        ObjectType::Hull,
        ObjectType::Tank,
        ObjectType::LoadingCondition,
        ObjectType::CargoTank,
        ObjectType::BallastTank,
        ObjectType::SlopTank,
    ];

    /// The object type of a key, or `None` for a key OVCD does not declare.
    pub fn from_key(key: &str) -> Option<ObjectType> {
        ObjectType::ALL.into_iter().find(|o| o.key() == key)
    }

    pub const fn key(self) -> &'static str {
        match self {
            ObjectType::Option => "option",
            ObjectType::Hull => "hull",
            ObjectType::Tank => "tank",
            ObjectType::LoadingCondition => "loading-condition",
            ObjectType::CargoTank => "cargo-tank",
            ObjectType::BallastTank => "ballast-tank",
            ObjectType::SlopTank => "slop-tank",
        }
    }
}

/// One entry of the Measure registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasureDef {
    pub id: &'static str,
    pub unit: &'static str,
    pub object: ObjectType,
    pub value: ValueKind,
    pub args: &'static [ArgDef],
    /// The method, cited: what the Measure computes and how.
    pub method: &'static str,
    pub method_status: MethodStatus,
}

const fn num(name: &'static str) -> ArgDef {
    ArgDef { name, text: false }
}

/// The computed Measures of the core. Input Measures (`input.<name>`) are added by the caller, which
/// knows the inputs and their units.
pub static MEASURES: &[MeasureDef] = &[
    MeasureDef {
        id: "gz.area",
        unit: "m.rad",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[num("from_deg"), num("to_deg")],
        method: "Trapezoid area under the GZ curve on the 1 degree grid, from from_deg to to_deg",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "gz.max_from",
        unit: "m",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[num("from_deg")],
        method: "Largest GZ on the 1 degree grid at a heel of from_deg or more",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "gz.angle_of_max",
        unit: "deg",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[],
        method: "Heel of the largest GZ on the 1 degree grid, 0 to 60 degrees; the first of equal values",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "gm0",
        unit: "m",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[],
        method: "Initial metacentric height of the box section, less the free surface correction",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.vc",
        unit: "m3",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Gross volume of one tank (cylinder and heads, inside the steel) plus its dome, as IGC 2.4.1.1 defines Vc",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.side_gap",
        unit: "m",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Clear gap from the moulded side to the outer face of the insulation, as the arrangement places the tank",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.bottom_gap",
        unit: "m",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Clear height from the moulded base to the outer face of the insulation: double bottom plus support height",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.insulation",
        unit: "m",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Insulation thickness of the tank",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.wall_thickness",
        unit: "mm",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "The thinner of the shell and head walls after forming, with the corrosion allowance (pressure-vessel formula, paywalled source)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.weld_factor",
        unit: "-",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "The welded joint efficiency factor the wall formula used",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.allowable_stress",
        unit: "MPa",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "The allowable stress f the wall formula used",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.rm",
        unit: "MPa",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Rm of the tank material: specified minimum tensile strength at room temperature (IGC 4.18.1.3)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.re",
        unit: "MPa",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "Re of the tank material: specified minimum yield stress at room temperature (IGC 4.18.1.3)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "stability.condition_ok",
        unit: "",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Bool,
        args: &[],
        method: "The engine's stability check of this loading condition: every IS Code 2008 Part A 2.2 criterion holds",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "check.ok",
        unit: "",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[ArgDef {
            name: "name",
            text: true,
        }],
        method: "The verdict of the engine's own check of this name, as the result's check list gives it",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "separation.cofferdam",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "Cofferdam width at the aft end of the cargo area, as the gas separation used it",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "separation.aft",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "Aft separation from the cargo area: the greater of the cofferdam and the opening setback",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "separation.fwd",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "Space between the cargo tanks and the forward end bulkhead of the hold",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "separation.tank_end",
        unit: "",
        object: ObjectType::Option,
        value: ValueKind::Text,
        args: &[],
        method: "The shape of the tank ends at the hold end bulkhead: plane or domed",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.material",
        unit: "",
        object: ObjectType::Tank,
        value: ValueKind::Text,
        args: &[],
        method: "The tank material key",
        method_status: MethodStatus::Unverified,
    },
];

/// A Measure of the core registry.
pub fn measure_def(id: &str) -> Option<&'static MeasureDef> {
    MEASURES.iter().find(|m| m.id == id)
}

impl MeasureDef {
    pub fn info(&self) -> MeasureInfo {
        MeasureInfo {
            unit: self.unit.to_string(),
            object: self.object.key().to_string(),
            value: self.value,
            args: self.args.to_vec(),
            method_status: self.method_status,
        }
    }
}

/// The formula shapes OVCD registers with the kernel: id, operands, coefficients, and the operands
/// whose unit is not the limit's (the coefficients convert them).
pub static FORMULA_SHAPES: &[FormulaShape] = &[
    // `of - less`: a value less an input (IGC 2.4.1.2 side distance less the insulation).
    shape("less-input", &["of", "less"], &[], &[]),
    // `max(lo, min(hi, a + x * b / c))`: "a + b·x, the lesser of that and hi, not less than lo"
    // (MARPOL Annex I Reg. 19.3.1: w = 0.5 + DW/20,000 or 2.0 m, whichever is the lesser, at least
    // 1.0 m).
    shape("clamp", &["x"], &["a", "b", "c", "lo", "hi"], &["x"]),
    // `max(lo, a + x * b / c)`: the same with no upper limit (MARPOL Annex I Reg. 19.6.2).
    shape("at-least", &["x"], &["a", "b", "c", "lo"], &["x"]),
    // `max(min(a + x * b / c, hi), of) - less`: the greater of a capped value and `of`, less an
    // input (IGC 2.4.1.2 bottom distance: the greater of B/15 or 2 m, whichever is less, and d,
    // less the insulation).
    shape(
        "max-capped-less",
        &["x", "of", "less"],
        &["a", "b", "c", "hi"],
        &["x"],
    ),
    // `min(rm / a, re / b)`: the lesser of two ratios (IGC 4.23.3.1, f = the lesser of Rm/A or
    // Re/B).
    shape("lesser-ratio", &["rm", "re", "a", "b"], &[], &["a", "b"]),
];

const fn shape(
    id: &'static str,
    operands: &'static [&'static str],
    coefficients: &'static [&'static str],
    free_unit_operands: &'static [&'static str],
) -> FormulaShape {
    FormulaShape {
        id,
        operands,
        coefficients,
        free_unit_operands,
        value: shape_value,
    }
}

fn shape_value(
    shape: &str,
    coefficients: &[(String, f64)],
    operand: &mut dyn FnMut(&str) -> Result<f64, String>,
) -> Result<f64, String> {
    formula_value(shape, coefficients, operand)
}

/// The value of a formula shape. The order of operations is the text's (`a + x * b / c`, then the
/// upper limit, then the lower), so a Rule written from the text stays bit-identical to a hand
/// calculation of the same text.
pub fn formula_value(
    shape: &str,
    coefficients: &[(String, f64)],
    mut operand: impl FnMut(&str) -> Result<f64, String>,
) -> Result<f64, String> {
    let c = |name: &str| -> Result<f64, String> {
        coefficients
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, x)| *x)
            .ok_or_else(|| format!("formula `{shape}` needs coefficient `{name}`"))
    };
    match shape {
        "less-input" => {
            let of = operand("of")?;
            Ok(of - operand("less")?)
        }
        "clamp" => {
            let v = c("a")? + operand("x")? * c("b")? / c("c")?;
            let (lo, hi) = (c("lo")?, c("hi")?);
            let v = if v < hi { v } else { hi };
            Ok(if v > lo { v } else { lo })
        }
        "at-least" => {
            let v = c("a")? + operand("x")? * c("b")? / c("c")?;
            let lo = c("lo")?;
            Ok(if v > lo { v } else { lo })
        }
        "max-capped-less" => {
            // The order of `py_max(py_min(beam / 15.0, 2.0), d_vc) - ins_m` in the gas arrangement.
            let v = c("a")? + operand("x")? * c("b")? / c("c")?;
            let hi = c("hi")?;
            let capped = if hi < v { hi } else { v };
            let of = operand("of")?;
            let greater = if of > capped { of } else { capped };
            Ok(greater - operand("less")?)
        }
        "lesser-ratio" => {
            // The order of `igc_allowable_stress` in `pressure_vessel`.
            let by_tensile = operand("rm")? / operand("a")?;
            let by_yield = operand("re")? / operand("b")?;
            Ok(if by_yield < by_tensile {
                by_yield
            } else {
                by_tensile
            })
        }
        other => Err(format!("unknown formula shape `{other}`")),
    }
}

/// Evaluate the selected sets on one Option, with OVCD's formula shapes
/// ([`arc_rules::eval::evaluate`]).
pub fn evaluate(sets: &[&RuleSet], source: &dyn MeasureSource) -> Trace {
    arc_rules::eval::evaluate(sets, FORMULA_SHAPES, source)
}

/// The failing rows of [`evaluate`], with OVCD's formula shapes ([`arc_rules::eval::failures`]).
pub fn failures(sets: &[&RuleSet], source: &dyn MeasureSource) -> Vec<Failure> {
    arc_rules::eval::failures(sets, FORMULA_SHAPES, source)
}
