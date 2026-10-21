//! ASME Section VIII / IGC Code pressure vessel wall thickness. Port of `core/pressure_vessel.py`.
//!
//! Where Python raises, the port returns a [`PyError`] with the same exception class and message.

use alloc::format;
use alloc::string::{String, ToString};

/// A Python exception: its class name and message. The wall-thickness maths can raise a
/// `ZeroDivisionError` (division by exactly zero), and `math.ceil` raises `ValueError` on NaN and
/// `OverflowError` on infinity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyError {
    pub kind: &'static str,
    pub message: String,
}

impl PyError {
    fn value(message: String) -> Self {
        Self {
            kind: "ValueError",
            message,
        }
    }
}

/// Mechanical and physical properties of a tank material.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TankMaterialProperties {
    pub name: &'static str,
    /// f, allowable stress (MPa): not more than [`igc_allowable_stress`], the lesser of Rm/A and
    /// Re/B (IGC 4.23.3.1).
    pub allowable_stress: f64,
    /// Rm, the specified minimum tensile strength at room temperature (MPa; IGC 4.18.1.3).
    pub tensile_strength: f64,
    /// Re, the specified minimum yield stress at room temperature, or the 0.2 % proof stress where
    /// the steel has no defined yield (MPa; IGC 4.18.1.3).
    pub yield_strength: f64,
    /// A and B, the least stress factors of IGC 4.23.3.1 for the steel's class: 3 and 1.5 for
    /// nickel and carbon-manganese steels, 3.5 and 1.5 for austenitic steels.
    pub factor_a: f64,
    pub factor_b: f64,
    /// Where Rm and Re come from.
    pub strength_source: &'static str,
    /// The allowable stress the engine used before the IGC check (issue #313), for a saved file
    /// that keeps it. Only S690QL1 differs: 285 MPa, Rm/2.7, the inland basis of the published
    /// river LPG tank.
    pub published_design_stress: f64,
    /// Material density (t/m^3)
    pub density: f64,
    /// IGC Code minimum thickness after forming (mm)
    pub min_thickness: f64,
    /// Corrosion allowance (mm)
    pub corrosion_allowance: f64,
    pub description: &'static str,
}

/// `TANK_MATERIAL_DATABASE`: (key, properties). The first three are in the Python dict's insertion
/// order, with the values of the original engine; issue #313 added the cited Rm and Re behind their
/// allowable stresses. `S690QL1` was added for the Type C tank defaults by cargo (issue #292).
pub static TANK_MATERIAL_DATABASE: [(&str, TankMaterialProperties); 4] = [
    (
        "304L",
        TankMaterialProperties {
            name: "304L Stainless Steel",
            allowable_stress: 113.0,
            tensile_strength: 485.0,
            yield_strength: 170.0,
            factor_a: 3.5,
            factor_b: 1.5,
            strength_source: OUTOKUMPU_ASTM,
            published_design_stress: 113.0,
            density: 7.93,
            min_thickness: 3.0,
            corrosion_allowance: 0.0,
            description: "Austenitic stainless, LNG/LPG service to -196\u{b0}C",
        },
    ),
    (
        "316L",
        TankMaterialProperties {
            name: "316L Stainless Steel",
            allowable_stress: 113.0,
            tensile_strength: 485.0,
            yield_strength: 170.0,
            factor_a: 3.5,
            factor_b: 1.5,
            strength_source: OUTOKUMPU_ASTM,
            published_design_stress: 113.0,
            density: 7.98,
            min_thickness: 3.0,
            corrosion_allowance: 0.0,
            description: "Austenitic stainless, enhanced corrosion resistance",
        },
    ),
    (
        "9Ni",
        TankMaterialProperties {
            name: "9% Nickel Steel",
            allowable_stress: 230.0,
            tensile_strength: 690.0,
            yield_strength: 585.0,
            factor_a: 3.0,
            factor_b: 1.5,
            strength_source: "ThyssenKrupp, 5 % and 9 % Ni steel plates (Cambridge LINK archive, item 95), \
                table of international standards: ASTM A 553 Type I, under 50 mm, Re over 585 MPa, Rm 690 to 825 MPa",
            published_design_stress: 230.0,
            density: 7.85,
            min_thickness: 5.0,
            corrosion_allowance: 0.5,
            description: "ASTM A553 Type I, LNG service to -196\u{b0}C",
        },
    ),
    // Issue #292. Density, minimum thickness and corrosion allowance are not in the published river
    // LPG tank paper: they take the values of the other carbon and nickel steels above (7.85 t/m3,
    // 5 mm, 0.5 mm), an estimate. Issue #313: the allowable stress is the IGC 4.23.3.1 limit, Rm/3
    // = 256.7 MPa (Re/1.5 = 460 MPa is higher). The paper's Rm/2.7 = 285 MPa, an inland basis, is
    // above that limit; it stays only as the stress of a file saved before.
    (
        "S690QL1",
        TankMaterialProperties {
            name: "S690QL1 quenched and tempered steel",
            allowable_stress: 770.0 / 3.0,
            tensile_strength: 770.0,
            yield_strength: 690.0,
            factor_a: 3.0,
            factor_b: 1.5,
            strength_source: "Senjanovic et al., Structure design of cargo tanks in river liquefied gas \
                carriers, DESIGN 2006: S690QL1, Re 690 and Rm 770 MPa. Macsteel S690QL data sheet (EN 10025-6): \
                3 to 50 mm, minimum yield 690 MPa, tensile 770 to 940 MPa",
            published_design_stress: 285.0,
            density: 7.85,
            min_thickness: 5.0,
            corrosion_allowance: 0.5,
            description: "High-strength C-Mn steel (Re 690, Rm 770 MPa), pressurised LPG tanks",
        },
    ),
];

/// Rm and Re of the austenitic steels: the ASTM minimums at room temperature.
const OUTOKUMPU_ASTM: &str = "Outokumpu, Steel grades, properties and global standards: ASTM minimum \
    values at room temperature, S30403 (304L) and S31603 (316L), Rp0.2 170 MPa, Rm 485 MPa";

/// The IGC 4.23.3.1 limit on the allowable stress of a Type C tank: f = the lesser of Rm/A and
/// Re/B (MPa). The database stress of each steel is not more than this; 304L and 316L hold 113 MPa,
/// the 113.3 MPa of Re/1.5 rounded down, as the original engine had it.
pub fn igc_allowable_stress(m: &TankMaterialProperties) -> f64 {
    let by_tensile = m.tensile_strength / m.factor_a;
    let by_yield = m.yield_strength / m.factor_b;
    if by_yield < by_tensile {
        by_yield
    } else {
        by_tensile
    }
}

/// Default weld joint efficiency E in Python's signatures.
pub const DEFAULT_WELD_EFFICIENCY: f64 = 0.95;

/// Look up tank material properties by key.
pub fn get_tank_material(material_key: &str) -> Result<&'static TankMaterialProperties, PyError> {
    match TANK_MATERIAL_DATABASE
        .iter()
        .find(|(key, _)| *key == material_key)
    {
        Some((_, props)) => Ok(props),
        None => {
            let mut valid = String::new();
            for (i, (key, _)) in TANK_MATERIAL_DATABASE.iter().enumerate() {
                if i > 0 {
                    valid.push_str(", ");
                }
                valid.push_str(key);
            }
            Err(PyError::value(format!(
                "Unknown tank material '{material_key}'. Valid: {valid}"
            )))
        }
    }
}

/// Python's float `/`: an exactly-zero divisor raises `ZeroDivisionError`.
fn div(a: f64, b: f64) -> Result<f64, PyError> {
    if b == 0.0 {
        return Err(PyError {
            kind: "ZeroDivisionError",
            message: "float division by zero".to_string(),
        });
    }
    Ok(a / b)
}

/// Python's `max(a, b)`: the first of equal items; `b` only if `b > a`.
fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// `math.ceil(x)` as a float: NaN and infinity raise, as in Python.
fn py_ceil(x: f64) -> Result<f64, PyError> {
    if x.is_nan() {
        return Err(PyError::value(
            "cannot convert float NaN to integer".to_string(),
        ));
    }
    if x.is_infinite() {
        return Err(PyError {
            kind: "OverflowError",
            message: "cannot convert float infinity to integer".to_string(),
        });
    }
    Ok(libm::ceil(x))
}

/// ASME UG-27 circumferential stress formula for cylindrical shells: t = P R / (f E - 0.6 P).
pub fn cylinder_wall_thickness(
    design_pressure_mpa: f64,
    inner_radius_mm: f64,
    allowable_stress_mpa: f64,
    weld_efficiency: f64,
) -> Result<f64, PyError> {
    let (p, r, f, e) = (
        design_pressure_mpa,
        inner_radius_mm,
        allowable_stress_mpa,
        weld_efficiency,
    );
    div(p * r, f * e - 0.6 * p)
}

/// Hemispherical head thickness: t = P R / (2 f E - 0.2 P). Not UG-27, which is the cylindrical
/// shell paragraph (issue #433); the ASME head paragraph is not in the rule-source library, so
/// none is cited.
pub fn hemisphere_head_thickness(
    design_pressure_mpa: f64,
    inner_radius_mm: f64,
    allowable_stress_mpa: f64,
    weld_efficiency: f64,
) -> Result<f64, PyError> {
    let (p, r, f, e) = (
        design_pressure_mpa,
        inner_radius_mm,
        allowable_stress_mpa,
        weld_efficiency,
    );
    div(p * r, 2.0 * f * e - 0.2 * p)
}

/// 2:1 ellipsoidal head thickness: t = P D / (2 f E - 0.2 P). Not UG-27, which is the cylindrical
/// shell paragraph (issue #433); the ASME head paragraph is not in the rule-source library, so
/// none is cited.
pub fn ellipsoidal_head_thickness(
    design_pressure_mpa: f64,
    inner_diameter_mm: f64,
    allowable_stress_mpa: f64,
    weld_efficiency: f64,
) -> Result<f64, PyError> {
    let (p, d, f, e) = (
        design_pressure_mpa,
        inner_diameter_mm,
        allowable_stress_mpa,
        weld_efficiency,
    );
    div(p * d, 2.0 * f * e - 0.2 * p)
}

/// The cylinder shell thickness the pressure-vessel formula asks for (mm), before the corrosion
/// allowance, the rounding to 0.5 mm and the IGC minimum: two passes of the ASME UG-27 shell
/// formula, the first on the outer radius and the second on the inner radius it gave. The radius
/// is the steel outer one, so the caller gives the steel outer diameter (the diameter without any
/// insulation).
pub fn required_cylinder_thickness_mm(
    outer_diameter_m: f64,
    design_pressure_mpa: f64,
    allowable_stress_mpa: f64,
    weld_efficiency: f64,
) -> Result<f64, PyError> {
    let outer_radius_mm = outer_diameter_m * 1000.0 / 2.0;
    let (p, f, e) = (design_pressure_mpa, allowable_stress_mpa, weld_efficiency);
    // First pass: approximate with inner radius ~ outer radius.
    let t_cyl = cylinder_wall_thickness(p, outer_radius_mm, f, e)?;
    // Second pass: refine with the corrected inner radius.
    cylinder_wall_thickness(p, outer_radius_mm - t_cyl, f, e)
}

/// Required wall thicknesses (mm) for the cylinder and the heads: `(cylinder, head)`.
///
/// Two passes on the cylinder, corrosion allowance, rounding up to 0.5 mm, then the IGC minimum.
/// `head_type` is `"hemisphere"` or `"ellipsoidal_2to1"`.
pub fn calculate_wall_thicknesses(
    outer_diameter_m: f64,
    design_pressure_mpa: f64,
    material_key: &str,
    head_type: &str,
    weld_efficiency: f64,
) -> Result<(f64, f64), PyError> {
    calculate_wall_thicknesses_with(
        outer_diameter_m,
        design_pressure_mpa,
        material_key,
        head_type,
        weld_efficiency,
        None,
    )
}

/// As [`calculate_wall_thicknesses`], with an allowable stress of the designer's own (MPa) instead
/// of the material's. The material still gives the minimum thickness and the corrosion allowance.
pub fn calculate_wall_thicknesses_with(
    outer_diameter_m: f64,
    design_pressure_mpa: f64,
    material_key: &str,
    head_type: &str,
    weld_efficiency: f64,
    allowable_stress_mpa: Option<f64>,
) -> Result<(f64, f64), PyError> {
    let mat = get_tank_material(material_key)?;
    let outer_radius_mm = outer_diameter_m * 1000.0 / 2.0;
    let f = allowable_stress_mpa.unwrap_or(mat.allowable_stress);
    let p = design_pressure_mpa;
    let e = weld_efficiency;

    // First pass: approximate with inner radius ~ outer radius.
    let t_cyl = cylinder_wall_thickness(p, outer_radius_mm, f, e)?;

    // Second pass: refine with the corrected inner radius.
    let inner_radius_mm = outer_radius_mm - t_cyl;
    let mut t_cyl = cylinder_wall_thickness(p, inner_radius_mm, f, e)?;

    // Head thickness using the corrected inner geometry.
    let inner_diameter_mm = 2.0 * inner_radius_mm;
    let mut t_head = match head_type {
        "hemisphere" => hemisphere_head_thickness(p, inner_radius_mm, f, e)?,
        "ellipsoidal_2to1" => ellipsoidal_head_thickness(p, inner_diameter_mm, f, e)?,
        other => return Err(PyError::value(format!("Unknown head type: {other}"))),
    };

    // Add corrosion allowance.
    t_cyl += mat.corrosion_allowance;
    t_head += mat.corrosion_allowance;

    // Round up to the nearest 0.5 mm (standard plate sizing).
    t_cyl = py_ceil(t_cyl * 2.0)? / 2.0;
    t_head = py_ceil(t_head * 2.0)? / 2.0;

    // Enforce IGC Code minimums.
    t_cyl = py_max(t_cyl, mat.min_thickness);
    t_head = py_max(t_head, mat.min_thickness);

    Ok((t_cyl, t_head))
}
