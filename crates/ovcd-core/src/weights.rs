//! Component weight breakdown for the ATB barge. Port of `core/weights.py`.
//!
//! Behaviour is copied exactly. `max` keeps Python's first-of-equal rule, every `**` is `libm::pow`, and
//! every expression keeps Python's evaluation order so the rounding is the same.

use alloc::string::ToString;
use alloc::vec::Vec;

use libm::{fabs, pow, sqrt};

use crate::hull::HullGeometry;
use crate::models::{BargeInputs, TankResult, WeightBreakdown};
use crate::pressure_vessel::PyError;
use crate::tanks::none_operand_error;

/// Python `max(a, b)`: the first of two equal items; `b` only if `b > a`.
fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// `inputs.wing_tank_width > 0`: Python raises `TypeError` when the field is still `None`.
fn wing_tank_width_positive(inputs: &BargeInputs) -> Result<bool, PyError> {
    match inputs.wing_tank_width {
        Some(w) => Ok(w > 0.0),
        None => Err(PyError {
            kind: "TypeError",
            message: "'>' not supported between instances of 'NoneType' and 'int'".to_string(),
        }),
    }
}

/// The length of the tank zone: `loa - bow_non_tank_length - stern_non_tank_length`.
fn tank_zone_length(inputs: &BargeInputs) -> Result<f64, PyError> {
    let bow = inputs.bow_non_tank_length.ok_or_else(none_operand_error)?;
    let stern = inputs
        .stern_non_tank_length
        .ok_or_else(none_operand_error)?;
    Ok(inputs.loa - bow - stern)
}

/// The outer hull plate thicknesses, in mm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlateThicknesses {
    pub bottom_mm: f64,
    pub side_mm: f64,
    pub deck_mm: f64,
}

/// The outer hull steel, in tonnes: plating by part, the internal structure and the total.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HullPlating {
    pub bottom: f64,
    pub side: f64,
    pub deck: f64,
    pub rake: f64,
    /// Frames, floors, longitudinals and bulkheads: the plating times `structure_factor - 1`.
    pub structure: f64,
    pub total: f64,
}

/// The outer hull steel from the plating areas and thicknesses, plus a structure factor for the
/// internal framing. Any hull with plating (the gas barge today, a tank barge later) uses it.
pub fn hull_plating(
    hull: &HullGeometry,
    plates: &PlateThicknesses,
    steel_density: f64,
    structure_factor: f64,
) -> HullPlating {
    // Convert plate thicknesses from mm to m.
    let t_bottom = plates.bottom_mm / 1000.0;
    let t_side = plates.side_mm / 1000.0;
    let t_deck = plates.deck_mm / 1000.0;

    let w_bottom = hull.bottom_area() * t_bottom * steel_density;
    let w_side = hull.side_area() * t_side * steel_density;
    let w_deck = hull.deck_area() * t_deck * steel_density;

    // Rake plating: average of bottom and side thickness.
    let t_rake = (t_bottom + t_side) / 2.0;
    let w_rake = (hull.bow_rake_area() + hull.stern_rake_area()) * t_rake * steel_density;

    // Plating subtotal.
    let w_plating = w_bottom + w_side + w_deck + w_rake;

    // Internal structure (frames, floors, longitudinals, bulkheads).
    let w_structure = w_plating * (structure_factor - 1.0);

    HullPlating {
        bottom: w_bottom,
        side: w_side,
        deck: w_deck,
        rake: w_rake,
        structure: w_structure,
        total: w_plating + w_structure,
    }
}

/// One mass of the lightship with its centre (issue #348): `lcg_m` from the aft end of the hull,
/// `kcg_m` above the keel.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MassItem {
    pub label: &'static str,
    pub mass_t: f64,
    pub lcg_m: f64,
    pub kcg_m: f64,
}

/// The sum of the masses (t).
pub fn total_mass(items: &[MassItem]) -> f64 {
    items.iter().map(|i| i.mass_t).sum()
}

/// The sum of mass x KCG (t.m), for a KG.
pub fn vertical_moment(items: &[MassItem]) -> f64 {
    items.iter().map(|i| i.mass_t * i.kcg_m).sum()
}

/// The covering above the moulded depth D of a gas hull with a wing tank (issue #410): a
/// non-structural weather covering, weighed as lightship items each with its own centre. Only the
/// parts above D count; the hull steel (`steel_rate_per_cbm` x LOA x B x D) holds what is below.
/// So a user who sets D over the tanks makes the covering structural, and the list is empty.
///
/// - the wing plate, at its own level, when it stands above D;
/// - the side plating from D up to the wing plate, at mid-height;
/// - the weather shield top, at the tank top plus the clearance;
/// - the weather shield sides and ends, at mid-height between the higher of the wing plate and D
///   and the shield top.
///
/// The plates take the same thicknesses as the bottom-up steel. The LCGs are from the aft end:
/// the wing plate's tank-zone part at the tank zone centre and its full-beam parts in the bow and
/// stern zones; the side plating at mid-length; the shield at the tank zone centre.
pub fn covering_items(inputs: &BargeInputs, tanks: &TankResult) -> Result<Vec<MassItem>, PyError> {
    let mut items = Vec::new();
    if !(wing_tank_width_positive(inputs)? && inputs.wing_tank_height_ratio < 1.0) {
        return Ok(items);
    }
    let rho = inputs.hull_steel_density;
    let d = inputs.depth;
    let db = inputs.double_bottom_height.unwrap_or(0.0);
    let tank_od = tanks.outer_diameter;
    let tank_r = tank_od / 2.0;
    let cargo_tank_bottom = db + inputs.tank_support_height;
    let wing_tank_top = cargo_tank_bottom + inputs.wing_tank_height_ratio * tank_od;
    let tank_zone_length = tank_zone_length(inputs)?;
    let stern = inputs.stern_non_tank_length.unwrap_or(0.0);
    let bow = inputs.bow_non_tank_length.unwrap_or(0.0);
    let tank_zone_centre = stern + tank_zone_length / 2.0;

    if wing_tank_top > d {
        // The wing plate: the beam less the tank cut-outs in the tank zone, the full beam in the
        // bow and stern zones (as `calculate_weights` weighs it).
        let dy = wing_tank_top - (cargo_tank_bottom + tank_r);
        let chord_width = if fabs(dy) < tank_r {
            2.0 * sqrt(pow(tank_r, 2.0) - pow(dy, 2.0))
        } else {
            0.0
        };
        let t_deck = inputs.deck_plate_thickness / 1000.0;
        let net_width = inputs.beam - (inputs.n_tanks_wide as f64) * chord_width;
        let m_zone = net_width * tank_zone_length * t_deck * rho;
        let m_stern = inputs.beam * stern * t_deck * rho;
        let m_bow = inputs.beam * bow * t_deck * rho;
        let mass = m_zone + m_stern + m_bow;
        let moment =
            m_zone * tank_zone_centre + m_stern * stern / 2.0 + m_bow * (inputs.loa - bow / 2.0);
        items.push(MassItem {
            label: "wing plate",
            mass_t: mass,
            lcg_m: if mass > 0.0 {
                moment / mass
            } else {
                inputs.loa / 2.0
            },
            kcg_m: wing_tank_top,
        });
        // The side plating from D up to the wing plate: two sides, full LOA.
        let t_side = inputs.side_plate_thickness / 1000.0;
        items.push(MassItem {
            label: "side plating to the wing plate",
            mass_t: 2.0 * (wing_tank_top - d) * inputs.loa * t_side * rho,
            lcg_m: inputs.loa / 2.0,
            kcg_m: (d + wing_tank_top) / 2.0,
        });
    }

    if inputs.weather_shield_thickness > 0.0 {
        let t_shield = inputs.weather_shield_thickness / 1000.0;
        let shield_top = cargo_tank_bottom + tank_od + inputs.weather_shield_clearance;
        if shield_top > d {
            items.push(MassItem {
                label: "weather shield top",
                mass_t: inputs.beam * tank_zone_length * t_shield * rho,
                lcg_m: tank_zone_centre,
                kcg_m: shield_top,
            });
            // The sides and ends stand on the wing plate, or on the deck where D is higher.
            let foot = py_max(wing_tank_top, d);
            let height = shield_top - foot;
            if height > 0.0 {
                let area = 2.0 * height * tank_zone_length + 2.0 * inputs.beam * height;
                items.push(MassItem {
                    label: "weather shield sides and ends",
                    mass_t: area * t_shield * rho,
                    lcg_m: tank_zone_centre,
                    kcg_m: (foot + shield_top) / 2.0,
                });
            }
        }
    }
    Ok(items)
}

/// Calculate the component weight breakdown, in tonnes.
///
/// Hull steel has two figures. The bottom-up one is built from plating areas and thicknesses, plus a
/// structure factor for internal framing (floors, frames, longitudinals, bulkheads), with the wing
/// plate and the weather shield (`steel_bottom_up`); the capital budget prices it. The top-down one
/// is `steel_rate_per_cbm` times the moulded volume to D (`steel_top_down`); the lightship, the
/// piping and the outfitting use it. The covering above D joins the lightship as its own items
/// (`covering_items`, issue #410). `steel_delta` is top-down less bottom-up (issue #401).
pub fn calculate_weights(
    inputs: &BargeInputs,
    hull: &HullGeometry,
    tanks: &TankResult,
) -> Result<WeightBreakdown, PyError> {
    let steel_density = inputs.hull_steel_density; // t/m^3
    let t_side = inputs.side_plate_thickness / 1000.0;

    // --- Hull plating weights ---
    let HullPlating {
        bottom: w_bottom,
        side: w_side,
        deck: w_deck,
        rake: w_rake,
        structure: w_structure,
        total: w_hull_total,
    } = hull_plating(
        hull,
        &PlateThicknesses {
            bottom_mm: inputs.bottom_plate_thickness,
            side_mm: inputs.side_plate_thickness,
            deck_mm: inputs.deck_plate_thickness,
        },
        steel_density,
        inputs.structure_factor,
    );

    // --- Tank weights (from tank calculation) ---
    let w_tank_shells = tanks.tank_shell_weight;
    // Internal stiffeners, ring frames, support saddles/stools.
    let w_tank_internals = w_tank_shells * (inputs.tank_structure_factor - 1.0);
    let w_tank_total = w_tank_shells + w_tank_internals;
    let w_insulation = tanks.insulation_weight;

    // --- Wing plate (horizontal steel shelf at wing tank top level) ---
    // Full beam x full LOA, minus cutouts where tanks pass through in the tank zone.
    let mut w_wing_plate = 0.0;
    if wing_tank_width_positive(inputs)? && inputs.wing_tank_height_ratio < 1.0 {
        let db = inputs.double_bottom_height.unwrap_or(0.0);
        let tank_od = tanks.outer_diameter;
        let tank_r = tank_od / 2.0;
        let cargo_tank_bottom = db + inputs.tank_support_height;
        let wing_tank_top = cargo_tank_bottom + inputs.wing_tank_height_ratio * tank_od;
        let tank_center_y = cargo_tank_bottom + tank_r;

        // Chord width of the tank at plate elevation (cutout in the tank zone only).
        let dy = wing_tank_top - tank_center_y;
        let chord_width = if fabs(dy) < tank_r {
            2.0 * sqrt(pow(tank_r, 2.0) - pow(dy, 2.0))
        } else {
            0.0
        };

        let tank_zone_length = tank_zone_length(inputs)?;
        let non_tank_length = inputs.loa - tank_zone_length; // bow + stern zones
        let t_deck_plate = inputs.deck_plate_thickness / 1000.0;

        // In the tank zone: beam minus chord cutouts per tank abreast.
        let net_plate_width_tz = inputs.beam - (inputs.n_tanks_wide as f64) * chord_width;
        let w_plate_tank_zone =
            net_plate_width_tz * tank_zone_length * t_deck_plate * steel_density;

        // In the non-tank zones: full beam (no tank cutouts).
        let w_plate_non_tank = inputs.beam * non_tank_length * t_deck_plate * steel_density;

        w_wing_plate = w_plate_tank_zone + w_plate_non_tank;

        // Side plating extension from deck to wing plate level (full LOA).
        let side_extension_height = wing_tank_top - inputs.depth;
        if side_extension_height > 0.0 {
            // Two sides x extension height x LOA x side plate thickness.
            let w_side_extension =
                2.0 * side_extension_height * inputs.loa * t_side * steel_density;
            w_wing_plate += w_side_extension;
        }
    }

    // --- Weather shield (enclosure above wing plate, around exposed tank) ---
    let mut w_weather_shield = 0.0;
    if inputs.weather_shield_thickness > 0.0
        && wing_tank_width_positive(inputs)?
        && inputs.wing_tank_height_ratio < 1.0
    {
        let db = inputs.double_bottom_height.unwrap_or(0.0);
        let tank_od = tanks.outer_diameter;
        let cargo_tank_bottom = db + inputs.tank_support_height;
        let wing_tank_top = cargo_tank_bottom + inputs.wing_tank_height_ratio * tank_od;
        let cargo_tank_top = cargo_tank_bottom + tank_od;
        let shield_top = cargo_tank_top + inputs.weather_shield_clearance;
        let shield_height = shield_top - wing_tank_top;
        let tank_zone_length = tank_zone_length(inputs)?;
        let t_shield = inputs.weather_shield_thickness / 1000.0;

        // Top plate: beam x tank zone length.
        let top_plate_area = inputs.beam * tank_zone_length;
        // Side walls: 2 x shield height x tank zone length.
        let side_wall_area = 2.0 * shield_height * tank_zone_length;
        // Forward and aft bulkheads: 2 x beam x shield height.
        let end_wall_area = 2.0 * inputs.beam * shield_height;

        let total_shield_area = top_plate_area + side_wall_area + end_wall_area;
        w_weather_shield = total_shield_area * t_shield * steel_density;
    }

    // --- Top-down vs bottom-up steel ---
    // Issue #410: the hull is weighed to the moulded depth D only. What stands above D (the wing
    // plate, the side plating up to it, the weather shield) is a non-structural covering: it goes
    // into the lightship as its own items (`covering_items`), not as hull girder.
    let moulded_depth = inputs.depth;
    let moulded_volume = inputs.loa * inputs.beam * moulded_depth;
    let steel_top_down = inputs.steel_rate_per_cbm * moulded_volume;
    let covering = total_mass(&covering_items(inputs, tanks)?);

    // Bottom-up: hull steel only (excluding tanks, which are separate).
    let steel_bottom_up = w_hull_total + w_wing_plate + w_weather_shield;
    let steel_delta = steel_top_down - steel_bottom_up;

    // --- Piping and machinery (based on top-down steel + tanks) ---
    let w_piping = inputs.piping_factor * (steel_top_down + w_tank_total);

    // --- Outfitting (percentage of steel + tanks) ---
    let w_outfitting = inputs.outfitting_factor * (steel_top_down + w_tank_total);

    // --- Lightship total (top-down hull steel + bottom-up tanks + insulation + piping + outfitting
    // + the covering above D) ---
    let lightship =
        steel_top_down + w_tank_total + w_insulation + w_piping + w_outfitting + covering;

    Ok(WeightBreakdown {
        hull_bottom_plating: w_bottom,
        hull_side_plating: w_side,
        hull_deck_plating: w_deck,
        hull_rake_plating: w_rake,
        hull_internal_structure: w_structure,
        hull_steel_total: w_hull_total,
        tank_shells: w_tank_shells,
        tank_internals_supports: w_tank_internals,
        tank_total: w_tank_total,
        insulation: w_insulation,
        wing_plate: w_wing_plate,
        weather_shield: w_weather_shield,
        piping_machinery: w_piping,
        outfitting: w_outfitting,
        // Consumables are deadweight, not lightship: an explicit zero allowance for now.
        stores: 0.0,
        moulded_volume,
        moulded_depth,
        steel_top_down,
        steel_bottom_up,
        steel_delta,
        lightship,
    })
}
