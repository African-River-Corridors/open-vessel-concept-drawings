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
    /// One assumed damage of a liquid bulk hull (issue #341): the flooded compartments of an extent.
    DamageCase,
    /// One assumed damage of MARPOL Annex I regulation 28 (issue #341): its extents reach the cargo tanks.
    MarpolDamageCase,
    /// The mean oil outflow of a liquid bulk hull (MARPOL Annex I regulation 23, issue #341): one object.
    OilOutflow,
}

impl ObjectType {
    pub const ALL: [ObjectType; 10] = [
        ObjectType::Option,
        ObjectType::Hull,
        ObjectType::Tank,
        ObjectType::LoadingCondition,
        ObjectType::CargoTank,
        ObjectType::BallastTank,
        ObjectType::SlopTank,
        ObjectType::DamageCase,
        ObjectType::MarpolDamageCase,
        ObjectType::OilOutflow,
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
            ObjectType::DamageCase => "damage-case",
            ObjectType::MarpolDamageCase => "marpol-damage-case",
            ObjectType::OilOutflow => "oil-outflow",
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
        id: "gz.area_to_downflooding",
        unit: "m.rad",
        object: ObjectType::LoadingCondition,
        value: ValueKind::Number,
        args: &[num("from_deg"), num("to_deg")],
        method: "Trapezoid area under the GZ curve on the 1 degree grid from from_deg to to_deg, or to the angle of down-flooding the designer gave where that is less (the last part with GZ interpolated linearly); none when the angle is not above from_deg",
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
        id: "hull.double_side_width",
        unit: "m",
        object: ObjectType::Hull,
        value: ValueKind::Number,
        args: &[],
        method: "Width of the double side of a liquid bulk hull: the layout's, the same along the tank block (the distance from the side shell to the longitudinal bulkhead of the cargo tanks)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "hull.double_bottom_height",
        unit: "m",
        object: ObjectType::Hull,
        value: ValueKind::Number,
        args: &[],
        method: "Height of the double bottom of a liquid bulk hull: the layout's, the same along the tank block",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "hull.lbh",
        unit: "m3",
        object: ObjectType::Hull,
        value: ValueKind::Number,
        args: &[],
        method: "L x B x H as ADN 2025 9.3.2.11.1 (a) defines it for the tank capacity table: L the overall length of the hull, B the extreme breadth, H the moulded depth within the cargo area",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "cargo-tank.volume",
        unit: "m3",
        object: ObjectType::CargoTank,
        value: ValueKind::Number,
        args: &[],
        method: "Gross moulded volume of one cargo tank of the liquid bulk layout (a port or starboard tank beside a centreline bulkhead is a tank); slop and ballast tanks are other objects",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "cargo-tank.length",
        unit: "m",
        object: ObjectType::CargoTank,
        value: ValueKind::Number,
        args: &[],
        method: "Length of one cargo tank of the liquid bulk layout, along the hull",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "cargo-tank.vessel_lbh",
        unit: "m3",
        object: ObjectType::CargoTank,
        value: ValueKind::Number,
        args: &[],
        method: "L x B x H of the hull the cargo tank is in (the value of hull.lbh), so a Rule on one cargo tank can read the table of ADN 2025 9.3.2.11.1 (a) on the same object",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "cargo-tank.vessel_length",
        unit: "m",
        object: ObjectType::CargoTank,
        value: ValueKind::Number,
        args: &[],
        method: "The overall length of the hull the cargo tank is in, so a Rule on one cargo tank can read the length limit of ADN 2025 9.3.2.11.1 (d) on the same object",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "option.adn_vessel_type",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Text,
        args: &[],
        method: "The ADN Tank Vessel Type of the Cargo (N_open, N_closed or C), derived from its Crude Grade by the ADN 2025 3.2.3.3 flowchart; an unknown flag gives C",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "option.deadweight",
        unit: "t",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The deadweight of the run: the displacement at the reported draught less the lightship, so the weight-bound or volume-bound cargo plus the stores and fuel",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "cargo-tank.marpol_length_ratio",
        unit: "-",
        object: ObjectType::CargoTank,
        value: ValueKind::Number,
        args: &[],
        method: "The length of the cargo tank over the longest tank MARPOL Annex I regulation 23.3.2 allows: 10 m or the formula of 23.3.2.1 to 23.3.2.3 (by the bulkheads of the tanks abreast and bi/B), whichever is the greater; at most 1 meets it",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "oil_outflow.ratio",
        unit: "-",
        object: ObjectType::OilOutflow,
        value: ValueKind::Number,
        args: &[],
        method: "The mean oil outflow parameter OM of MARPOL Annex I regulation 23 (damaged tank method: side and bottom damage probabilities of 23.8 and 23.9, outflow by pressure balance of 23.7, combined by 23.5) over the largest OM that 23.3.1 allows for the cargo volume: at most 1 meets it",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "option.cargo_capacity_m3",
        unit: "m3",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "Gross moulded volume of the cargo tanks of a liquid bulk layout, slop tanks excluded",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "option.slop_capacity_share",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The gross volume of the slop tanks over the gross volume of the cargo and slop tanks together; zero with no slop tank",
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
        id: "wheelhouse.blind_distance",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[ArgDef {
            name: "condition",
            text: true,
        }],
        method: "The distance ahead of the bow up to which the helmsman's view of the water is blocked (light or laden): where the ray from the eye over the cargo or tank top at the forward end of the cargo zone meets the waterline, trimmed where the trim is known",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "powering.speed_through_water_kmh",
        unit: "km/h",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[ArgDef {
            name: "environment",
            text: true,
        }],
        method: "The speed through the water at the installed power, laden, in the named environment (the powering's service speed, 1 kn = 1.852 km/h)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "powering.speed_over_design",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The lowest service speed of any environment at the installed power, over the Design Speed",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "wheelhouse.lift_enough",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "True when the lift the Blind Distance needs is within the maximum lift of the wheelhouse",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "powering.depth_over_draught",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The depth of the design channel over the loaded draught (h/T)",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "propulsion.diameter_over_max",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The propeller diameter over the largest the Stern Form allows (D_max)",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "propulsion.immersion_over_limit",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The shaft axis submergence over the propeller diameter, at the lightest aft draught of the loading conditions, over the limit of the type (0.76 open, 0.71 ducted; Beveridge 1971, a guide). The axis is half a diameter above the base",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "propulsion.blade_load_over_limit",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The delivered power of one unit at the Installed Power per m2 of disc, over the limit of the type (400 kW/m2 open, 500 ducted; Veth rating basis)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "freeboard.laden",
        unit: "m",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "The Freeboard at the laden Draught: the Moulded Depth less the Draught of the laden result, amidships and level, the deck plating not added (ICLL regulation 3(8) measures the freeboard assigned from the deck line down to the load line)",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "propulsion.redundancy_28_04",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "ES-TRIN 28.04(1)(a) from the inputs: a length of 110 m or less, or two or more propulsors with a bow thruster, or one propulsor with a bow thruster. Equal power, independence and a thruster that works unladen are assumed, as the model has no such detail",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "wheelhouse.field_of_view",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether the helmsman's field of unobstructed vision meets 7.02(3); the model has no window geometry",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "wheelhouse.raised_stability",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether the raised wheelhouse leaves the stability met; the model has no stability case with it raised",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "formation.coupling_force",
        unit: "kN",
        object: ObjectType::Option,
        value: ValueKind::Number,
        args: &[],
        method: "ES-TRIN ESI-III-6 2.1: F_SB = 270 x P_B x L_S / B_S x 10^-3 kN for a pusher and its pushed craft, with P_B the pusher's installed power and L_S, B_S the pusher's length and beam, from the Formation powering",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "formation.separate_vessels",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "ABS Steel Barges 5-3-1/1.3: the tug and the barge are classed as two vessels; an approval item, the model does not assess either against class",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "formation.notch_wing_walls",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "ABS Steel Barges 5-3-1/11.3: the analysis of the barge wing walls at a notch (torsional, primary and secondary shear and bending); an approval item, the model has no structure model for it",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "openings.front_clear",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "True when the typed deckhouse front has no entrance, air inlet or opening (the input says whether it has one)",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "arrangement.hold_space_segregation",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether the hold spaces are segregated from machinery, accommodation, service spaces, control stations, stores and domestic water tanks, by the means IGC 3.1.1 and 3.1.2 allow; the model has no such spaces",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "arrangement.accommodation_outside_cargo_area",
        unit: "-",
        object: ObjectType::Option,
        value: ValueKind::Bool,
        args: &[],
        method: "Whether no accommodation space, service space or control station is within the cargo area; the model does not place them",
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
        id: "tank.design_pressure",
        unit: "MPa",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "The design vapour pressure (gauge) the wall formula used",
        method_status: MethodStatus::Verified,
    },
    MeasureDef {
        id: "tank.dynamic_pressure_term",
        unit: "MPa",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "A C (rho_r)^1.5 of IGC 4.23.1.2: A = 0.00185 (sigma_m / 55)^2 with sigma_m taken as the allowable stress the wall formula used; C the greatest of the tank outer diameter, 0.75 times it and 0.45 times the overall tank length; rho_r the liquid density at the design temperature over 1 t/m3",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "tank.design_temperature",
        unit: "C",
        object: ObjectType::Tank,
        value: ValueKind::Number,
        args: &[],
        method: "The lowest design temperature of the tank",
        method_status: MethodStatus::Verified,
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
        id: "stability.damage",
        unit: "",
        object: ObjectType::DamageCase,
        value: ValueKind::Bool,
        args: &[],
        method: "The damaged hull floats where the heel at the stage of equilibrium is at most 12 degrees and, beyond it, the righting lever reaches 0.05 m with an area of 0.0065 m.rad up to 27 degrees (ADN 2025 9.3.2.15.2 and 9.3.3.15.2). Lost buoyancy method on the Sections Table, final stage of flooding only: the intermediate stages, the non-watertight openings and the flooding of undamaged compartments are not modelled",
        method_status: MethodStatus::Unverified,
    },
    MeasureDef {
        id: "stability.damage_marpol",
        unit: "",
        object: ObjectType::MarpolDamageCase,
        value: ValueKind::Bool,
        args: &[],
        method: "The damaged hull floats where the heel at the stage of equilibrium is at most 25 degrees and the righting lever stays positive over 20 degrees beyond it, reaching 0.1 m with an area of 0.0175 m.rad within that range (MARPOL Annex I regulation 28.3.2 and 28.3.3). Lost buoyancy method on the Sections Table, final stage of flooding, for the extents of regulation 28.2; a damaged cargo tank is flooded at 95 % and the oil keeps its mass",
        method_status: MethodStatus::Unverified,
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
