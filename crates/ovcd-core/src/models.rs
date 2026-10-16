//! Data models for barge design inputs and outputs. Port of `core/models.py`.
//!
//! Holds the shared types and no entry points of its own. Each Python dataclass is one struct
//! with the same fields in the same order. [`BargeInputs::new`] gives the dataclass defaults;
//! [`BargeInputs::post_init`] is Python's `__post_init__` and the caller runs it after setting
//! fields, exactly as Python runs it after `__init__`. A Python `Optional[float]` is an
//! `Option<f64>`.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::aft_arrangement::AftArrangementResult;
use crate::gas_separation::GasSeparationResult;
use crate::pressure_vessel::{PyError, get_tank_material};
use crate::protective_location::ProtectiveLocationResult;

/// All user-provided inputs for ATB LNG/LPG barge design.
#[derive(Debug, Clone, PartialEq)]
pub struct BargeInputs {
    // --- Principal dimensions ---
    /// Length overall (m)
    pub loa: f64,
    /// Moulded beam (m)
    pub beam: f64,
    /// Moulded depth (m)
    pub depth: f64,
    /// Maximum design draft (m)
    pub max_draft: f64,
    /// Max height above waterline allowed (m)
    pub vertical_clearance: f64,
    /// "IGF", "IGC_1G", "IGC_2G", "IGC_3G"
    pub vessel_type: String,

    // --- Rake geometry ---
    /// Horizontal length of bow bottom rake (m)
    pub bow_rake_length: f64,
    /// Vertical rise of bow rake (m), defaults to depth
    pub bow_rake_rise: Option<f64>,
    /// Horizontal length of stern bottom rake (m)
    pub stern_rake_length: f64,
    /// Vertical rise of stern rake (m), defaults to depth
    pub stern_rake_rise: Option<f64>,

    // --- Tank arrangement constraints ---
    /// Forward non-tank zone as fraction of LOA
    pub bow_non_tank_pct: f64,
    /// Aft non-tank zone as fraction of LOA
    pub stern_non_tank_pct: f64,
    /// (m) computed from pct; set explicitly to override
    pub bow_non_tank_length: Option<f64>,
    /// (m) computed from pct; set explicitly to override
    pub stern_non_tank_length: Option<f64>,
    /// Wing ballast tank width per side (m); None = auto (side_clearance - gap)
    pub wing_tank_width: Option<f64>,
    /// Gap between wing tank and cargo tank, per side (m)
    pub wing_tank_gap: f64,
    /// Wing tank height as fraction of cargo tank OD
    pub wing_tank_height_ratio: f64,
    /// Weather shield plate thickness (mm); 0 = no shield
    pub weather_shield_thickness: f64,
    /// Weather shield top above cargo tank top (m)
    pub weather_shield_clearance: f64,
    /// Dome protrusion above cargo tank top (m); extends beyond shield
    pub dome_height: f64,
    /// Total side clearance per side (m); auto = wing_tank_width + wing_tank_gap
    pub side_clearance: Option<f64>,
    /// Gap between tanks across the beam (m)
    pub tank_gap_transverse: f64,
    /// Gap between tanks along the length (m)
    pub tank_gap_longitudinal: f64,
    /// Number of tanks abreast
    pub n_tanks_wide: i64,
    /// Number of tank rows along the length
    pub n_tanks_long: i64,

    // --- Tank properties ---
    /// Fixed wall thickness (mm); None = auto from pressure vessel formula
    pub tank_wall_thickness: Option<f64>,
    /// "hemisphere" or "ellipsoidal_2to1"
    pub tank_head_type: String,
    /// Material key: "304L", "316L", "9Ni", "S690QL1"
    pub tank_material: String,
    /// Allowable stress (MPa) the thickness formula uses; `None`: the material's own.
    pub tank_allowable_stress: Option<f64>,
    /// Tank material density (t/m^3)
    pub tank_material_density: f64,
    /// Design pressure (MPa)
    pub design_pressure: f64,
    /// Weld joint efficiency (0.85-1.0)
    pub weld_efficiency: f64,
    /// Internal stiffeners, supports, saddles as multiple of shell weight
    pub tank_structure_factor: f64,

    // --- Insulation ---
    /// Insulation thickness (mm)
    pub insulation_thickness: f64,
    /// Insulation density (kg/m^3)
    pub insulation_density: f64,

    // --- Cargo ---
    /// "LNG", "LNG_ethane", "LPG_propane", "LPG_butane", "LPG_mix", or "custom"
    pub cargo_type: String,
    /// Custom cargo density (kg/m^3), overrides cargo_type lookup
    pub cargo_density_override: Option<f64>,
    /// Maximum fill level (fraction)
    pub fill_ratio: f64,

    // --- Hull structural assumptions ---
    /// Bottom plating thickness (mm)
    pub bottom_plate_thickness: f64,
    /// Side plating thickness (mm)
    pub side_plate_thickness: f64,
    /// Deck plating thickness (mm)
    pub deck_plate_thickness: f64,
    /// Internal structure weight as multiple of plating
    pub structure_factor: f64,
    /// Carbon steel density (t/m^3)
    pub hull_steel_density: f64,

    // --- Other ---
    /// Top-down steel weight rate (t/m^3 of moulded volume)
    pub steel_rate_per_cbm: f64,
    /// Block coefficient Cb (top-down assumption)
    pub block_coefficient: f64,
    /// Water density (t/m^3)
    pub water_density: f64,
    /// Required underkeel clearance (m)
    pub underkeel_clearance: f64,
    /// Outfitting as fraction of (steel + tanks)
    pub outfitting_factor: f64,
    /// Piping and machinery as fraction of (hull + tank weight)
    pub piping_factor: f64,
    /// Diesel, fresh water, provisions (tonnes)
    pub stores_weight: f64,

    // --- Tank support ---
    /// Height of tank saddles/supports above inner bottom (m)
    pub tank_support_height: f64,
    /// Double bottom height (m); None = auto from protective location
    pub double_bottom_height: Option<f64>,

    // --- Arrangement, DRAWING-ONLY pass-through ---
    pub aft_arrangement: Option<AftArrangementResult>,
    pub separation: Option<GasSeparationResult>,

    // --- ATB tug (for drawing only, not cargo calculation) ---
    /// Tug length extending aft from barge stern (m)
    pub tug_length: f64,
    /// Pusher-tug beam (m)
    pub tug_beam: f64,
    /// V-notch cutout depth into stern (m)
    pub notch_depth: f64,
}

impl BargeInputs {
    /// The dataclass defaults, with the five required principal dimensions. Run [`post_init`]
    /// after changing fields.
    ///
    /// [`post_init`]: BargeInputs::post_init
    pub fn new(loa: f64, beam: f64, depth: f64, max_draft: f64, vertical_clearance: f64) -> Self {
        Self {
            loa,
            beam,
            depth,
            max_draft,
            vertical_clearance,
            vessel_type: "IGC_2G".to_string(),
            bow_rake_length: 0.0,
            bow_rake_rise: None,
            stern_rake_length: 0.0,
            stern_rake_rise: None,
            bow_non_tank_pct: 0.10,
            stern_non_tank_pct: 0.15,
            bow_non_tank_length: None,
            stern_non_tank_length: None,
            wing_tank_width: None,
            wing_tank_gap: 0.6,
            wing_tank_height_ratio: 0.66,
            weather_shield_thickness: 6.0,
            weather_shield_clearance: 1.5,
            dome_height: 2.0,
            side_clearance: None,
            tank_gap_transverse: 1.0,
            tank_gap_longitudinal: 2.0,
            n_tanks_wide: 1,
            n_tanks_long: 2,
            tank_wall_thickness: Some(25.0),
            tank_head_type: "hemisphere".to_string(),
            tank_material: "9Ni".to_string(),
            tank_allowable_stress: None,
            tank_material_density: 7.85,
            design_pressure: 0.5,
            weld_efficiency: 0.95,
            tank_structure_factor: 1.30,
            insulation_thickness: 350.0,
            insulation_density: 60.0,
            cargo_type: "LNG_ethane".to_string(),
            cargo_density_override: None,
            fill_ratio: 0.95,
            bottom_plate_thickness: 12.0,
            side_plate_thickness: 10.0,
            deck_plate_thickness: 10.0,
            structure_factor: 1.4,
            hull_steel_density: 7.85,
            steel_rate_per_cbm: 0.10,
            block_coefficient: 0.85,
            water_density: 1.0,
            underkeel_clearance: 0.5,
            outfitting_factor: 0.035,
            piping_factor: 0.04,
            stores_weight: 200.0,
            tank_support_height: 0.5,
            double_bottom_height: Some(1.0),
            aft_arrangement: None,
            separation: None,
            tug_length: 25.0,
            tug_beam: 9.1,
            notch_depth: 3.0,
        }
    }

    /// Python's `__post_init__`: fill the fields left as `None`. `Err` is the `ValueError` the
    /// material lookup raises for an unknown material key.
    pub fn post_init(&mut self) -> Result<(), PyError> {
        if self.bow_rake_rise.is_none() {
            self.bow_rake_rise = Some(self.depth);
        }
        if self.stern_rake_rise.is_none() {
            self.stern_rake_rise = Some(self.depth);
        }
        // Compute non-tank lengths from percentages if not explicitly set.
        if self.bow_non_tank_length.is_none() {
            self.bow_non_tank_length = Some(self.bow_non_tank_pct * self.loa);
        }
        if self.stern_non_tank_length.is_none() {
            self.stern_non_tank_length = Some(self.stern_non_tank_pct * self.loa);
        }
        // Side clearance per side defaults to 1.0 m. The wing ballast tank fills the clearance
        // inboard of the cargo-tank gap.
        if self.side_clearance.is_none() {
            self.side_clearance = Some(1.0);
        }
        if self.wing_tank_width.is_none() {
            // Python: max(0.0, side_clearance - wing_tank_gap); the first of equal items wins.
            let v = self.side_clearance.unwrap_or(1.0) - self.wing_tank_gap;
            self.wing_tank_width = Some(if v > 0.0 { v } else { 0.0 });
        }
        // Sync material density from the database when auto-calculating thickness.
        if self.tank_wall_thickness.is_none() {
            let mat = get_tank_material(&self.tank_material)?;
            self.tank_material_density = mat.density;
        }
        Ok(())
    }
}

/// Results for tank arrangement and geometry.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TankResult {
    pub n_tanks_wide: i64,
    pub n_tanks_long: i64,
    pub n_tanks_total: i64,
    /// m
    pub outer_diameter: f64,
    /// m
    pub inner_diameter: f64,
    /// Overall tank length per tank including heads (m)
    pub overall_tank_length: f64,
    /// Cylindrical section length (m)
    pub cylinder_length: f64,
    /// Length of one head (m)
    pub head_length: f64,
    /// Interior volume per tank (m^3)
    pub volume_per_tank: f64,
    /// Cylinder interior volume per tank (m^3)
    pub volume_cylinder: f64,
    /// Both heads interior volume per tank (m^3)
    pub volume_heads: f64,
    /// Total interior volume all tanks (m^3)
    pub total_tank_volume: f64,
    /// Usable cargo volume at fill ratio (m^3)
    pub cargo_volume: f64,
    /// Weight of all tank shells (tonnes)
    pub tank_shell_weight: f64,
    /// Weight of all tank insulation (tonnes)
    pub insulation_weight: f64,
    /// Height of tank top above keel (m)
    pub tank_top_above_keel: f64,
    /// Cylinder wall thickness used (mm); Python default 0.0
    pub cylinder_wall_thickness: f64,
    /// Head wall thickness used (mm); Python default 0.0
    pub head_wall_thickness: f64,
    /// Design pressure used (MPa); Python default 0.0
    pub design_pressure: f64,
    /// Material key used (empty if fixed override)
    pub tank_material: String,
}

/// Component weight breakdown in tonnes.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct WeightBreakdown {
    pub hull_bottom_plating: f64,
    pub hull_side_plating: f64,
    pub hull_deck_plating: f64,
    pub hull_rake_plating: f64,
    pub hull_internal_structure: f64,
    pub hull_steel_total: f64,
    pub tank_shells: f64,
    pub tank_internals_supports: f64,
    pub tank_total: f64,
    pub insulation: f64,
    /// Horizontal wing plate (steel shelf at wing tank top)
    pub wing_plate: f64,
    /// Weather shield enclosure above wing plate
    pub weather_shield: f64,
    pub piping_machinery: f64,
    pub outfitting: f64,
    /// Diesel, fresh water, provisions
    pub stores: f64,
    /// LOA x beam x moulded depth (m^3)
    pub moulded_volume: f64,
    /// Effective moulded depth to wing plate (m)
    pub moulded_depth: f64,
    /// Top-down: rate x moulded volume (tonnes)
    pub steel_top_down: f64,
    /// Bottom-up: sum of all steel components (tonnes)
    pub steel_bottom_up: f64,
    /// Top-down minus bottom-up (tonnes)
    pub steel_delta: f64,
    pub lightship: f64,
}

/// Stability calculation results.
#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StabilityResult {
    /// Center of buoyancy above keel (m)
    pub KB: f64,
    /// Transverse metacentric radius (m)
    pub BM_transverse: f64,
    /// Center of gravity above keel (m)
    pub KG: f64,
    /// Metacentric height before free surface (m)
    pub GM_solid: f64,
    /// Free surface effect on GM (m)
    pub free_surface_correction: f64,
    /// Effective GM after free surface correction (m)
    pub GM_fluid: f64,
    /// Longitudinal metacentric radius (m)
    pub BM_longitudinal: f64,
}

/// Trim calculation results.
#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TrimResult {
    /// Longitudinal center of gravity from AP (m)
    pub LCG: f64,
    /// Longitudinal center of buoyancy from AP (m)
    pub LCB: f64,
    /// Trim (m), positive = bow down
    pub trim: f64,
    /// Draft at stern (m)
    pub draft_aft: f64,
    /// Draft at bow (m)
    pub draft_fwd: f64,
}

/// Complete barge design calculation results.
#[derive(Debug, Clone, PartialEq)]
pub struct BargeResults {
    /// Inputs echo
    pub inputs: BargeInputs,
    pub tanks: TankResult,
    pub cargo_type: String,
    /// t/m^3
    pub cargo_density: f64,
    /// tonnes
    pub cargo_mass: f64,
    pub weights: WeightBreakdown,
    /// tonnes
    pub displacement: f64,
    /// tonnes
    pub deadweight: f64,
    /// Equilibrium draft (m)
    pub draft: f64,
    /// m
    pub freeboard: f64,
    /// True if freeboard >= 0.3 m
    pub min_freeboard_ok: bool,
    /// Height above waterline to tank top, laden (m)
    pub air_draft: f64,
    /// True if air_draft <= vertical_clearance
    pub air_draft_ok: bool,
    pub stability: StabilityResult,
    /// True if GM_fluid > 0.15 m
    pub stability_ok: bool,
    pub trim: TrimResult,
    /// draft + underkeel clearance (m)
    pub total_depth_required: f64,
    /// Air draft, unladen (m); Python default 0.0
    pub air_draft_unladen: f64,
    /// The LIGHT equilibrium draft that unladen air draft is measured from; default 0.0
    pub light_draft: f64,
    /// Air draft with a lifting wheelhouse RAISED; default 0.0
    pub air_draft_navigating_unladen: f64,
    pub protective_location: Option<ProtectiveLocationResult>,
    /// Which constraints limit cargo capacity (Python `list`, default empty)
    pub binding_constraints: Vec<String>,
    /// Python `list`, default empty
    pub warnings: Vec<String>,
}
