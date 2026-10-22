//! Hydrostatic calculations: displacement, draft, stability and trim. Port of
//! `core/hydrostatics.py`.
//!
//! Behaviour is copied exactly. The bisection stops early on the first midpoint that is close
//! enough, as Python's does. Every sum keeps Python's left-to-right order so the rounding is the
//! same, including the free surface sum, which adds the same moment once per tank. Truthiness of
//! `double_bottom_height` follows Python: `None` and `0.0` are false, `NaN` is true.

use alloc::vec::Vec;

use alloc::string::ToString;

use libm::{cos, sin};

use crate::hull::hull_form::{HullError, HullFormInputs, HullFormKind, HullShape, generate};
use crate::models::{BargeInputs, StabilityResult, TankResult, TrimResult, WeightBreakdown};
use crate::pressure_vessel::PyError;
use crate::sections::{
    Equilibrium, Gravity, SectionsError, SectionsTable, Upright, Waterplane, draught_for_volume,
    equilibrium, gz_curve, immersed, upright,
};
use crate::stern_form::{ApplyError, SternGeometry};
use crate::tanks::{free_surface_moment_cylindrical, none_operand_error};

/// Find the draft at which displacement equals total weight (issue #448): the upright, level
/// draught of the hull's sections, by [`draught_for_volume`]. The block coefficient and the
/// tolerance of the 0.1.0 bisection are still taken and have no effect: the sections are the hull.
/// A weight the whole hull cannot float gives the bisection's upper bound, 1.5 x `max_draft`.
/// `ZeroDivisionError` when `water_density` is exactly zero, as in Python.
pub fn find_equilibrium_draft(
    hull: &SectionsTable,
    total_weight: f64,
    water_density: f64,
    max_draft: f64,
    _block_coefficient: f64,
    _tolerance: f64,
) -> Result<f64, PyError> {
    if water_density == 0.0 {
        return Err(PyError {
            kind: "ZeroDivisionError",
            message: "float division by zero".into(),
        });
    }
    match draught_for_volume(hull, total_weight / water_density) {
        Ok(t) => Ok(t),
        Err(SectionsError::Sinks) => Ok(max_draft * 1.5),
        Err(SectionsError::NotAfloat) => Ok(0.0),
        Err(e) => Err(PyError {
            kind: "ValueError",
            message: e.to_string(),
        }),
    }
}

/// The upright cut of the legacy entry points. A draught with no immersed volume gives zeros, with
/// the centre of buoyancy at mid-length, as the closed forms did; a NaN draught gives NaN.
fn upright_or_zero(hull: &SectionsTable, draft: f64, density: f64) -> Upright {
    if let Ok(u) = upright(hull, draft, density) {
        return u;
    }
    let v = if draft.is_nan() { f64::NAN } else { 0.0 };
    let mut im = immersed(hull, &Waterplane::upright(0.0));
    let (first, last) = (hull.sections[0].x, hull.sections[hull.sections.len() - 1].x);
    im.lcb_m = if draft.is_nan() {
        f64::NAN
    } else {
        (first + last) / 2.0
    };
    im.kb_m = v;
    Upright {
        draught_m: draft,
        immersed: im,
        displacement_t: v,
        bm_t_m: v,
        bm_l_m: v,
        km_t_m: v,
        km_l_m: v,
        tpc_t: v,
        mct_1cm_tm: v,
        cb: v,
        cm: v,
        cp: v,
        cwp: v,
    }
}

/// Intact stability (GM) at the equilibrium draft.
///
/// Legacy chain, kept for the Python API (issue #413). No Design uses it: the gas designs use
/// `gas_stability::assess` and the trim uses `trim`. Its limits: the KG items take the bottom-up
/// hull steel, so they do not sum to the lightship; one KG for all hull steel at half the depth.
/// The free surface takes the fill as a volume ratio (issue #413). Issue #448: the hull is its
/// Sections Table, so the hull form no longer counts twice.
///
/// `TypeError` when `double_bottom_height` is unset: Python adds it to a float.
pub fn calculate_stability(
    hull: &SectionsTable,
    inputs: &BargeInputs,
    tanks: &TankResult,
    weights: &WeightBreakdown,
    cargo_mass: f64,
    cargo_density_t: f64,
    draft: f64,
) -> Result<StabilityResult, PyError> {
    // Issue #448: the upright cut of the sections; the 0.1.0 double use of Cb goes.
    let u = upright_or_zero(hull, draft, inputs.water_density);
    let displacement = u.displacement_t;

    // --- KB ---
    let kb = u.immersed.kb_m;

    // --- BM ---
    let bm_t = u.bm_t_m;
    let bm_l = u.bm_l_m;

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

/// Longitudinal trim. x is measured from the stern (AP); positive trim is bow deeper.
///
/// Legacy chain, kept for the Python API (issue #413). No Design uses it; the designs use `trim`
/// with the gas stability's mass items. Issue #448: the hull is its Sections Table.
///
/// `TypeError` when a non-tank length is unset, as Python raises it.
pub fn calculate_trim(
    hull: &SectionsTable,
    inputs: &BargeInputs,
    _tanks: &TankResult,
    weights: &WeightBreakdown,
    cargo_mass: f64,
    draft: f64,
) -> Result<TrimResult, PyError> {
    // Issue #448: the upright cut of the sections.
    let u = upright_or_zero(hull, draft, inputs.water_density);
    let displacement = u.displacement_t;

    // --- LCB (from AP) ---
    let lcb = u.immersed.lcb_m;

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
    let bm_l = u.bm_l_m;

    let trim = if bm_l > 0.0 && displacement > 0.0 {
        let l_wp = u.immersed.l_wl_m;
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

// --- The hull from the Sections Table (issue #448, spec 0008) ---------------------------------

/// What a hull is made from: the main dimensions, the raked bow and the Stern Form. A bow rake of
/// no length is a vertical bow (the box).
#[derive(Debug, Clone, PartialEq)]
pub struct HullSpec {
    pub loa: f64,
    pub beam: f64,
    pub depth: f64,
    pub bow_rake_length_m: f64,
    pub bow_rake_rise_m: f64,
    pub stern: Option<SternGeometry>,
    /// Issue #449: bilge radius, sheer and camber.
    pub shape: HullShape,
}

/// Why a hull cannot be built.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildError {
    Hull(HullError),
    Stern(ApplyError),
}

impl core::fmt::Display for BuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BuildError::Hull(e) => write!(f, "{e}"),
            BuildError::Stern(e) => write!(f, "{e}"),
        }
    }
}

/// The hull as its Sections Table, with the hydrostatics the engine reads from it.
#[derive(Debug, Clone, PartialEq)]
pub struct Hull {
    pub spec: HullSpec,
    pub table: SectionsTable,
}

/// The free-trim floating position at heel 0, and the draughts at the ends and at mid-length (m,
/// measured square to the base line). Trim is positive by the bow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Float {
    pub equilibrium: Equilibrium,
    pub draught_aft_m: f64,
    pub draught_mid_m: f64,
    pub draught_fwd_m: f64,
    pub trim_m: f64,
}

/// The draughts of the hydrostatic table: 0.1 D to D in tenths.
pub const TABLE_ROWS: u32 = 10;

impl Hull {
    /// Write the Sections Table: the raked box of the spec, then the Stern Form's run. A bow rake
    /// longer than the hull, or rising above its depth, is cut to the hull: such a short hull is
    /// all rake (the box hull of the closed forms ignored the rake).
    pub fn build(spec: &HullSpec) -> Result<Self, BuildError> {
        let rake = spec.bow_rake_length_m.min(spec.loa);
        let form = if rake > 0.0 {
            HullFormKind::RakedBox {
                bow_rake_length_m: rake,
                bow_rake_rise_m: spec.bow_rake_rise_m.min(spec.depth),
                stern_rake_length_m: 0.0,
                stern_rake_rise_m: 0.0,
            }
        } else {
            HullFormKind::Box
        };
        let mut table = generate(&HullFormInputs {
            length_overall_m: spec.loa,
            beam_m: spec.beam,
            moulded_depth_m: spec.depth,
            form,
            shape: spec.shape.clone(),
        })
        .map_err(BuildError::Hull)?;
        // An offsets table or a ShipD hull holds its own stern (issues #451, #452).
        if let Some(g) = spec
            .stern
            .as_ref()
            .filter(|_| spec.shape.offsets.is_none() && spec.shape.shipd.is_none())
        {
            crate::stern_form::apply(g, &mut table).map_err(BuildError::Stern)?;
        }
        Ok(Self {
            spec: spec.clone(),
            table,
        })
    }

    /// The immersed volume below the upright, level waterline `draught` (m³).
    pub fn volume_at(&self, draught: f64) -> f64 {
        immersed(&self.table, &Waterplane::upright(draught)).volume_m3
    }

    /// The upright, level hydrostatics at `draught`.
    pub fn upright(&self, draught: f64, density: f64) -> Result<Upright, SectionsError> {
        upright(&self.table, draught, density)
    }

    /// The upright, level draught at which the hull displaces `mass_t`.
    pub fn level_draught(&self, mass_t: f64, density: f64) -> Result<f64, SectionsError> {
        draught_for_volume(&self.table, mass_t / density)
    }

    /// The free-trim floating position at heel 0 for a mass and its centre of gravity.
    pub fn float(&self, mass_t: f64, g: Gravity, density: f64) -> Result<Float, SectionsError> {
        let eq = equilibrium(&self.table, mass_t, g, 0.0, density)?;
        let wp = eq.waterplane;
        let theta = wp.trim_deg * (core::f64::consts::PI / 180.0);
        // Upright, a point (x, 0, z) is on the waterplane when -x sin θ + z cos θ = c.
        let at = |x: f64| (wp.c + x * sin(theta)) / cos(theta);
        let (aft, fwd) = (at(0.0), at(self.spec.loa));
        Ok(Float {
            equilibrium: eq,
            draught_aft_m: aft,
            draught_mid_m: at(self.spec.loa / 2.0),
            draught_fwd_m: fwd,
            trim_m: fwd - aft,
        })
    }

    /// The GZ curve with free trim at the given heels (degrees), less the free-surface correction.
    pub fn gz(
        &self,
        mass_t: f64,
        g: Gravity,
        free_surface_moment_tm: f64,
        density: f64,
        angles: impl IntoIterator<Item = i32>,
    ) -> Result<Vec<(i32, f64)>, SectionsError> {
        gz_curve(
            &self.table,
            mass_t,
            g,
            free_surface_moment_tm,
            density,
            angles,
        )
    }

    /// The upright hydrostatics at [`TABLE_ROWS`] draughts, 0.1 D to D.
    pub fn hydrostatic_table(&self, density: f64) -> Result<Vec<Upright>, SectionsError> {
        (1..=TABLE_ROWS)
            .map(|i| {
                self.upright(
                    self.spec.depth * f64::from(i) / f64::from(TABLE_ROWS),
                    density,
                )
            })
            .collect()
    }
}
