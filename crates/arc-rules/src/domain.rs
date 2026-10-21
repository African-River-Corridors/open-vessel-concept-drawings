//! What the caller declares (spec 0004, "Extension points"): its object types, its Measure
//! registry, its applicability vocabulary and its formula shapes. The kernel holds none of them.
//!
//! An object type is a key, such as `option` or `reach`. A Measure names the object type it is
//! computed for. The loader asks a [`Domain`] about every name a Rule Set file uses, so a typo is
//! refused at load time; the evaluator asks a [`crate::eval::MeasureSource`] for the values.

use alloc::string::String;
use alloc::vec::Vec;

/// What a Measure gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Number,
    Bool,
    Text,
}

/// Whether a Measure's method has been checked against a source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodStatus {
    Verified,
    Unverified,
}

impl MethodStatus {
    pub const fn key(self) -> &'static str {
        match self {
            MethodStatus::Verified => "verified",
            MethodStatus::Unverified => "unverified",
        }
    }
}

/// An argument a Measure takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgDef {
    pub name: &'static str,
    pub text: bool,
}

/// What the evaluator and the loader need to know of a Measure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasureInfo {
    pub unit: String,
    /// The key of the object type the Measure is computed for.
    pub object: String,
    pub value: ValueKind,
    pub args: Vec<ArgDef>,
    pub method_status: MethodStatus,
}

/// Whether an input of the applicability vocabulary has one value for the whole design, or one
/// value per object (a reach of a channel, say).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputScope {
    Design,
    Object,
}

/// An input an Applicability may name: its values and its scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputDef {
    pub values: Vec<String>,
    pub scope: InputScope,
}

/// The value of a formula shape: the shape id, the Rule's coefficients (sorted by name) and a
/// reader of its operands by name.
pub type FormulaFn = fn(
    shape: &str,
    coefficients: &[(String, f64)],
    operand: &mut dyn FnMut(&str) -> Result<f64, String>,
) -> Result<f64, String>;

/// A formula shape the caller registers: its operands (Measures, or table rows picked by a
/// Measure), its coefficients (numbers in the Rule Set file) and its value. A shape keeps the
/// order of operations of the text it codes, so a Rule stays bit-identical to a hand calculation.
#[derive(Debug, Clone, Copy)]
pub struct FormulaShape {
    pub id: &'static str,
    pub operands: &'static [&'static str],
    pub coefficients: &'static [&'static str],
    /// The operands whose unit is not the limit's: the coefficients convert them.
    pub free_unit_operands: &'static [&'static str],
    pub value: FormulaFn,
}

/// The coefficient `name` of a formula or table row, or why it is missing.
pub fn coefficient(shape: &str, coefficients: &[(String, f64)], name: &str) -> Result<f64, String> {
    coefficients
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, x)| *x)
        .ok_or_else(|| alloc::format!("formula `{shape}` needs coefficient `{name}`"))
}

/// The shape `id` of a list of shapes.
pub fn shape<'a>(shapes: &'a [FormulaShape], id: &str) -> Option<&'a FormulaShape> {
    shapes.iter().find(|s| s.id == id)
}

/// What the loader checks a Rule Set file against.
pub trait Domain {
    /// The registry entry of a Measure id, or `None` for an unknown id.
    fn measure(&self, id: &str) -> Option<MeasureInfo>;
    /// An input an Applicability may name, or `None` for one it cannot name.
    fn input(&self, name: &str) -> Option<InputDef>;
    /// The formula shapes.
    fn shapes(&self) -> &[FormulaShape];
    /// The object type of the design itself. A limit may read a Measure of this type from a Rule
    /// on any object.
    fn design_object(&self) -> &str;
    /// Where a user finds the Measure registry, for the message on an unknown Measure.
    fn measure_list_hint(&self) -> Option<&str> {
        None
    }
}
