//! Hydrostatic calculations: displacement, draft, stability and trim. Port of
//! `core/hydrostatics.py`.
//!
//! Behaviour is copied exactly. The bisection stops early on the first midpoint that is close
//! enough, as Python's does. Every sum keeps Python's left-to-right order so the rounding is the
//! same, including the free surface sum, which adds the same moment once per tank. Truthiness of
//! `double_bottom_height` follows Python: `None` and `0.0` are false, `NaN` is true.

use libm::fabs;

use crate::hull::HullGeometry;
use crate::models::{BargeInputs, StabilityResult, TankResult, TrimResult, WeightBreakdown};
use crate::pressure_vessel::PyError;
use crate::tanks::{free_surface_moment_cylindrical, none_operand_error};

/// Find the draft at which displacement equals total weight, by bisection.
///
/// The block coefficient accounts for the difference between the moulded hull volume and the
/// actual displacement volume. `ZeroDivisionError` when `water_density * block_coefficient` is
/// exactly zero, as in Python.
pub fn find_equilibrium_draft(
    hull: &HullGeometry,
    total_weight: f64,
    water_density: f64,
    max_draft: f64,
    block_coefficient: f64,
    tolerance: f64,
) -> Result<f64, PyError> {
    let divisor = water_density * block_coefficient;
    if divisor == 0.0 {
        return Err(PyError {
            kind: "ZeroDivisionError",
            message: "float division by zero".into(),
        });
    }
    let required_volume = total_weight / divisor;

    let mut lo = 0.0;
    // Allow some margin to detect overload.
    let mut hi = max_draft * 1.5;

    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        let vol = hull.displaced_volume(mid);
        if fabs(vol - required_volume) < tolerance * hull.beam {
            return Ok(mid);
        }
        if vol < required_volume {
            lo = mid;
        } else {
            hi = mid;
        }
    }

    Ok((lo + hi) / 2.0)
}

/// Intact stability (GM) at the equilibrium draft.
///
/// `TypeError` when `double_bottom_height` is unset: Python adds it to a float.
pub fn calculate_stability(
    hull: &HullGeometry,
    inputs: &BargeInputs,
    tanks: &TankResult,
    weights: &WeightBreakdown,
    cargo_mass: f64,
    cargo_density_t: f64,
    draft: f64,
) -> Result<StabilityResult, PyError> {
    let cb = inputs.block_coefficient;
    let displacement = hull.displaced_volume(draft) * inputs.water_density * cb;

    // --- KB ---
    let kb = hull.KB(draft);

    // --- BM ---
    let bm_t = hull.BM_transverse(draft);
    let bm_l = hull.BM_longitudinal(draft);

    // --- KG (centre of gravity above keel): weighted average of component centres ---
    // Hull steel: approximate CG at depth/2.
    let kg_hull = inputs.depth / 2.0;
    let w_hull = weights.hull_steel_total;

    // Tanks and insulation: CG at the centre of the tank.
    let double_bottom_height = inputs.double_bottom_height.ok_or(PyError {
        kind: "TypeError",
        message: "unsupported operand type(s) for +: 'NoneType' and 'float'".into(),
    })?;
    let kg_tanks = double_bottom_height + inputs.tank_support_height + tanks.outer_diameter / 2.0;
    let w_tanks = weights.tank_shells + weights.insulation;

    // Cargo: CG at the tank centre.
    let kg_cargo = kg_tanks;
    let w_cargo = cargo_mass;

    // Piping: approximate CG at deck level.
    let kg_piping = inputs.depth;
    let w_piping = weights.piping_machinery;

    // Outfitting: approximate CG at 0.75 * depth.
    let kg_outfitting = 0.75 * inputs.depth;
    let w_outfitting = weights.outfitting;

    // Stores: low in the hull. Python tests the double bottom height for truth.
    let kg_stores = if double_bottom_height != 0.0 {
        double_bottom_height / 2.0
    } else {
        inputs.depth * 0.25
    };
    let w_stores = weights.stores;

    let total_w = w_hull + w_tanks + w_cargo + w_piping + w_outfitting + w_stores;
    let kg = if total_w > 0.0 {
        (w_hull * kg_hull
            + w_tanks * kg_tanks
            + w_cargo * kg_cargo
            + w_piping * kg_piping
            + w_outfitting * kg_outfitting
            + w_stores * kg_stores)
            / total_w
    } else {
        inputs.depth / 2.0
    };

    // --- GM (solid) ---
    let gm_solid = kb + bm_t - kg;

    // --- Free surface correction: one moment per tank, added one at a time ---
    let mut total_fs_moment = 0.0;
    for _ in 0..tanks.n_tanks_total {
        let fs_moment = free_surface_moment_cylindrical(
            tanks.inner_diameter / 2.0,
            tanks.cylinder_length,
            inputs.fill_ratio,
            cargo_density_t,
        );
        total_fs_moment += fs_moment;
    }

    let fs_correction = if displacement > 0.0 {
        total_fs_moment / displacement
    } else {
        0.0
    };

    let gm_fluid = gm_solid - fs_correction;

    Ok(StabilityResult {
        KB: kb,
        BM_transverse: bm_t,
        KG: kg,
        GM_solid: gm_solid,
        free_surface_correction: fs_correction,
        GM_fluid: gm_fluid,
        BM_longitudinal: bm_l,
    })
}

/// Trim from mass items (issue #349): `items` are (mass t, LCG m from the stern). The LCG is the
/// mass-weighted centre; trim = (LCG - LCB) x L_wp / BM_L, the moment over MTC with MTC = Delta x
/// BM_L / L_wp, as [`calculate_trim`] has it; positive trim is bow deeper. No mass, or no
/// waterplane, gives no trim.
pub fn trim(hull: &HullGeometry, draft: f64, items: &[(f64, f64)]) -> TrimResult {
    trim_with_stern(hull, draft, items, None)
}

/// [`trim`] with the Stern Form's correction (issue #350): the volume the stern removes moves the
/// LCB forward, LCB = (V x LCB_box - M_removed) / (V - V_removed).
pub fn trim_with_stern(
    hull: &HullGeometry,
    draft: f64,
    items: &[(f64, f64)],
    stern: Option<&crate::stern_form::SternGeometry>,
) -> TrimResult {
    let mut lcb = hull.LCB(draft);
    if let Some(g) = stern {
        let (v_cut, m_cut) = g.removed(draft);
        let v = hull.displaced_volume(draft);
        if v - v_cut > 0.0 {
            lcb = (v * lcb - m_cut) / (v - v_cut);
        }
    }
    let total: f64 = items.iter().map(|(m, _)| m).sum();
    let lcg = if total > 0.0 {
        items.iter().map(|(m, x)| m * x).sum::<f64>() / total
    } else {
        lcb
    };
    let bm_l = hull.BM_longitudinal(draft);
    let trim = if bm_l > 0.0 && total > 0.0 {
        (lcg - lcb) * hull.waterplane_length(draft) / bm_l
    } else {
        0.0
    };
    TrimResult {
        LCG: lcg,
        LCB: lcb,
        trim,
        draft_aft: draft - trim / 2.0,
        draft_fwd: draft + trim / 2.0,
    }
}

/// Longitudinal trim. x is measured from the stern (AP); positive trim is bow deeper.
///
/// `TypeError` when a non-tank length is unset, as Python raises it.
pub fn calculate_trim(
    hull: &HullGeometry,
    inputs: &BargeInputs,
    _tanks: &TankResult,
    weights: &WeightBreakdown,
    cargo_mass: f64,
    draft: f64,
) -> Result<TrimResult, PyError> {
    let cb = inputs.block_coefficient;
    let displacement = hull.displaced_volume(draft) * inputs.water_density * cb;

    // --- LCB (from AP) ---
    let lcb = hull.LCB(draft);

    // --- LCG (from AP) ---
    // Hull steel: approximately at midship.
    let lcg_hull = inputs.loa / 2.0;
    let w_hull = weights.hull_steel_total;

    // Tanks, insulation and cargo: centred in the tank zone.
    // An unset length is a Python `TypeError`; `loa - bow` is evaluated before the sum.
    let bow_non_tank_length = inputs.bow_non_tank_length.ok_or_else(none_operand_error)?;
    let tank_zone_end = inputs.loa - bow_non_tank_length;
    let stern_non_tank_length = inputs.stern_non_tank_length.ok_or(PyError {
        kind: "TypeError",
        message: "unsupported operand type(s) for +: 'NoneType' and 'float'".into(),
    })?;
    let tank_zone_start = stern_non_tank_length;
    let tank_zone_center = (tank_zone_start + tank_zone_end) / 2.0;

    let w_tanks = weights.tank_shells + weights.insulation;
    let w_cargo = cargo_mass;

    // Piping: approximately at midship.
    let lcg_piping = inputs.loa / 2.0;
    let w_piping = weights.piping_machinery;

    // Outfitting and stores: in the aft zone.
    let lcg_outfitting = stern_non_tank_length / 2.0;
    let w_outfitting = weights.outfitting;
    let lcg_stores = stern_non_tank_length / 2.0;
    let w_stores = weights.stores;

    let total_w = w_hull + w_tanks + w_cargo + w_piping + w_outfitting + w_stores;
    let lcg = if total_w > 0.0 {
        (w_hull * lcg_hull
            + w_tanks * tank_zone_center
            + w_cargo * tank_zone_center
            + w_piping * lcg_piping
            + w_outfitting * lcg_outfitting
            + w_stores * lcg_stores)
            / total_w
    } else {
        inputs.loa / 2.0
    };

    // --- Trim ---
    let bm_l = hull.BM_longitudinal(draft);

    let trim = if bm_l > 0.0 && displacement > 0.0 {
        let l_wp = hull.waterplane_length(draft);
        (lcg - lcb) * l_wp / bm_l
    } else {
        0.0
    };

    // Positive trim = bow deeper.
    let draft_aft = draft - trim / 2.0;
    let draft_fwd = draft + trim / 2.0;

    Ok(TrimResult {
        LCG: lcg,
        LCB: lcb,
        trim,
        draft_aft,
        draft_fwd,
    })
}
